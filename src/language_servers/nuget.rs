use std::cmp::Ordering;

use zed_extension_api::{self as zed, http_client, serde_json, Result};

const ROSLYN_NUGET_FEED_INDEX: &str = "https://api.nuget.org/v3/index.json";

pub struct NuGetClient {
    package_base_address: Option<String>,
}

impl NuGetClient {
    pub fn new() -> Self {
        NuGetClient {
            package_base_address: None,
        }
    }

    fn ensure_package_base_address(&mut self) -> Result<String> {
        if let Some(ref base) = self.package_base_address {
            return Ok(base.clone());
        }

        let response = http_client::fetch(
            &http_client::HttpRequest::builder()
                .method(http_client::HttpMethod::Get)
                .url(ROSLYN_NUGET_FEED_INDEX)
                .redirect_policy(http_client::RedirectPolicy::FollowAll)
                .build()?,
        )?;

        let index: serde_json::Value = serde_json::from_slice(&response.body)
            .map_err(|e| format!("failed to parse NuGet service index: {e}"))?;

        let base_url = index["resources"]
            .as_array()
            .ok_or("invalid NuGet service index: missing 'resources' array")?
            .iter()
            .find(|r| {
                r["@type"]
                    .as_str()
                    .is_some_and(|t| t == "PackageBaseAddress/3.0.0")
            })
            .and_then(|r| r["@id"].as_str())
            .ok_or("PackageBaseAddress/3.0.0 not found in NuGet service index")?
            .trim_end_matches('/')
            .to_string();

        self.package_base_address = Some(base_url.clone());
        Ok(base_url)
    }

    pub fn get_latest_version(&mut self, package_id: &str) -> Result<String> {
        let base = self.ensure_package_base_address()?;
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

        let versions = body["versions"]
            .as_array()
            .ok_or_else(|| format!("no versions array for NuGet package '{package_id}'"))?;

        versions
            .iter()
            .filter_map(|v| v.as_str())
            .filter_map(NuGetVersion::parse)
            .max()
            .map(|v| v.raw)
            .ok_or_else(|| format!("no parseable versions found for NuGet package '{package_id}'"))
    }

    pub fn download_and_extract(
        &mut self,
        package_id: &str,
        version: &str,
        dest_dir: &str,
    ) -> Result<()> {
        let base = self.ensure_package_base_address()?;
        let lower_id = package_id.to_lowercase();
        let lower_version = version.to_lowercase();

        let url = format!("{base}/{lower_id}/{lower_version}/{lower_id}.{lower_version}.nupkg");

        zed::download_file(&url, dest_dir, zed::DownloadedFileType::Zip)
            .map_err(|e| format!("failed to download NuGet package '{package_id}' v{version}: {e}"))
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
