use std::{cmp::Ordering, collections::HashMap};

use zed_extension_api::{self as zed, http_client, serde_json, settings::LspSettings, Result};

const DEFAULT_NUGET_FEED_INDEX: &str = "https://api.nuget.org/v3/index.json";

pub struct NuGetClient {
    package_base_addresses: HashMap<String, String>,
}

fn source_from_settings(settings: Option<&serde_json::Value>) -> Result<String> {
    let Some(value) = settings.and_then(|settings| settings.get("nuget_source")) else {
        return Ok(DEFAULT_NUGET_FEED_INDEX.to_string());
    };
    let source = value
        .as_str()
        .ok_or("nuget_source must be an HTTP(S) NuGet v3 service index URL")?;
    let address = source
        .strip_prefix("https://")
        .or_else(|| source.strip_prefix("http://"))
        .ok_or("nuget_source must be an HTTP(S) NuGet v3 service index URL")?;
    if address.split('/').next().unwrap_or_default().is_empty()
        || source.chars().any(char::is_whitespace)
    {
        return Err("nuget_source must be an HTTP(S) NuGet v3 service index URL".to_string());
    }
    Ok(source.to_string())
}

pub fn remove_source_setting(settings: &mut serde_json::Value) {
    if let Some(object) = settings.as_object_mut() {
        object.remove("nuget_source");
    }
}

impl NuGetClient {
    pub fn new() -> Self {
        NuGetClient {
            package_base_addresses: HashMap::new(),
        }
    }

    fn source(worktree: &zed::Worktree, language_server_id: &str) -> Result<String> {
        let settings = LspSettings::for_worktree(language_server_id, worktree)?;
        source_from_settings(settings.settings.as_ref())
    }

    fn package_base_address(&mut self, source: &str) -> Result<String> {
        if let Some(base) = self.package_base_addresses.get(source) {
            return Ok(base.clone());
        }

        let response = http_client::fetch(
            &http_client::HttpRequest::builder()
                .method(http_client::HttpMethod::Get)
                .url(source)
                .redirect_policy(http_client::RedirectPolicy::FollowAll)
                .build()?,
        )?;

        let index: serde_json::Value = serde_json::from_slice(&response.body)
            .map_err(|e| format!("failed to parse NuGet service index from {source}: {e}"))?;

        let base_url = index["resources"]
            .as_array()
            .ok_or_else(|| format!("invalid NuGet service index at {source}: missing 'resources'"))?
            .iter()
            .find(|r| {
                r["@type"]
                    .as_str()
                    .is_some_and(|t| t.starts_with("PackageBaseAddress/3.0.0"))
            })
            .and_then(|r| r["@id"].as_str())
            .ok_or_else(|| format!("PackageBaseAddress/3.0.0 not found in {source}"))?
            .trim_end_matches('/')
            .to_string();

        self.package_base_addresses
            .insert(source.to_string(), base_url.clone());
        Ok(base_url)
    }

    fn latest_version_from_source(
        &mut self,
        source: &str,
        package_id: &str,
    ) -> Result<NuGetVersion> {
        let base = self.package_base_address(source)?;
        let lower_id = package_id.to_lowercase();

        let url = format!("{base}/{lower_id}/index.json");
        let response = http_client::fetch(
            &http_client::HttpRequest::builder()
                .method(http_client::HttpMethod::Get)
                .url(&url)
                .redirect_policy(http_client::RedirectPolicy::FollowAll)
                .build()?,
        )?;

        let body: serde_json::Value = serde_json::from_slice(&response.body)
            .map_err(|e| format!("failed to parse NuGet version index for '{package_id}': {e}"))?;

        body["versions"]
            .as_array()
            .ok_or_else(|| format!("no versions array for NuGet package '{package_id}'"))?
            .iter()
            .filter_map(|v| v.as_str())
            .filter_map(NuGetVersion::parse)
            .max()
            .ok_or_else(|| format!("no parseable versions found for NuGet package '{package_id}'"))
    }

    pub fn get_latest_version(
        &mut self,
        worktree: &zed::Worktree,
        language_server_id: &str,
        package_id: &str,
    ) -> Result<String> {
        let source = Self::source(worktree, language_server_id)?;
        self.latest_version_from_source(&source, package_id)
            .map(|version| version.raw)
            .map_err(|error| format!("failed to query NuGet source '{source}': {error}"))
    }

    pub fn download_and_extract(
        &mut self,
        worktree: &zed::Worktree,
        language_server_id: &str,
        package_id: &str,
        version: &str,
        dest_dir: &str,
    ) -> Result<()> {
        let lower_id = package_id.to_lowercase();
        let lower_version = version.to_lowercase();

        let source = Self::source(worktree, language_server_id)?;
        let base = self.package_base_address(&source)?;
        let url = format!("{base}/{lower_id}/{lower_version}/{lower_id}.{lower_version}.nupkg");
        zed::download_file(&url, dest_dir, zed::DownloadedFileType::Zip).map_err(|error| {
            format!("failed to download NuGet package '{package_id}' v{version} from '{source}': {error}")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_nuget_only_when_source_is_not_configured() {
        assert_eq!(
            source_from_settings(None).unwrap(),
            DEFAULT_NUGET_FEED_INDEX
        );
        assert_eq!(
            source_from_settings(Some(&serde_json::json!({"other": true}))).unwrap(),
            DEFAULT_NUGET_FEED_INDEX
        );
    }

    #[test]
    fn configured_feed_replaces_nuget_org() {
        for source in [
            "https://packages.example.com/v3/index.json",
            "http://localhost:8080/v3/index.json",
        ] {
            let settings = serde_json::json!({"nuget_source": source});
            assert_eq!(source_from_settings(Some(&settings)).unwrap(), source);
        }
    }

    #[test]
    fn invalid_configuration_errors_instead_of_falling_back() {
        for value in [
            serde_json::Value::Null,
            serde_json::json!(42),
            serde_json::json!(""),
            serde_json::json!("file:///packages"),
            serde_json::json!("https:///index.json"),
            serde_json::json!("https://packages.example.com/invalid url"),
        ] {
            let settings = serde_json::json!({"nuget_source": value});
            assert!(source_from_settings(Some(&settings)).is_err());
        }
    }

    #[test]
    fn extension_setting_is_not_forwarded_to_language_servers() {
        let mut settings = serde_json::json!({
            "nuget_source": "https://packages.example.com/v3/index.json",
            "other": {"enabled": true}
        });
        remove_source_setting(&mut settings);
        assert_eq!(settings, serde_json::json!({"other": {"enabled": true}}));
    }
}
#[derive(Debug, Clone)]
struct NuGetVersion {
    major: u64,
    minor: u64,
    patch: u64,
    revision: u64,
    prerelease: Option<String>,
    raw: String,
}

impl PartialEq for NuGetVersion {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for NuGetVersion {}

impl NuGetVersion {
    fn parse(input: &str) -> Option<Self> {
        let (core, prerelease) = match input.split_once('-') {
            Some((c, p)) => (c, Some(p.to_string())),
            None => (input, None),
        };

        let segments: Vec<u64> = core
            .split('.')
            .map(|s| s.parse::<u64>().ok())
            .collect::<Option<Vec<_>>>()?;

        let (major, minor, patch, revision) = match segments[..] {
            [major] => (major, 0, 0, 0),
            [major, minor] => (major, minor, 0, 0),
            [major, minor, patch] => (major, minor, patch, 0),
            [major, minor, patch, revision] => (major, minor, patch, revision),
            _ => return None,
        };

        Some(NuGetVersion {
            major,
            minor,
            patch,
            revision,
            prerelease,
            raw: input.to_string(),
        })
    }
}

fn cmp_prerelease_token(a: &str, b: &str) -> Ordering {
    match (a.parse::<u64>(), b.parse::<u64>()) {
        (Ok(na), Ok(nb)) => na.cmp(&nb),
        (Ok(_), Err(_)) => Ordering::Less,
        (Err(_), Ok(_)) => Ordering::Greater,
        (Err(_), Err(_)) => a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase()),
    }
}

impl Ord for NuGetVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
            .then(self.revision.cmp(&other.revision))
            .then_with(|| match (&self.prerelease, &other.prerelease) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(a), Some(b)) => {
                    let mut a_parts = a.split('.');
                    let mut b_parts = b.split('.');
                    loop {
                        match (a_parts.next(), b_parts.next()) {
                            (Some(at), Some(bt)) => {
                                let ord = cmp_prerelease_token(at, bt);
                                if ord != Ordering::Equal {
                                    return ord;
                                }
                            }
                            (None, Some(_)) => return Ordering::Less,
                            (Some(_), None) => return Ordering::Greater,
                            (None, None) => return Ordering::Equal,
                        }
                    }
                }
            })
    }
}

impl PartialOrd for NuGetVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
