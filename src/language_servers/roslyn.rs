use std::fs;

use zed_extension_api::{
    self as zed, serde_json::Map, settings::LspSettings, LanguageServerId, Result,
};

const REPO: &str = "SofusA/csharp-language-server";

pub struct Roslyn {
    cached_binary_path: Option<String>,
}

impl Roslyn {
    pub const LANGUAGE_SERVER_ID: &'static str = "roslyn";

    pub fn new() -> Self {
        Roslyn {
            cached_binary_path: None,
        }
    }

    pub fn language_server_cmd(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let binary_settings = LspSettings::for_worktree("roslyn", worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.binary);
        let binary_args = binary_settings
            .as_ref()
            .and_then(|binary_settings| binary_settings.arguments.clone());

        if let Some(path) = binary_settings.and_then(|binary_settings| binary_settings.path) {
            return Ok(zed::Command {
                command: path,
                args: binary_args.unwrap_or_default(),
                env: Default::default(),
            });
        }

        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(zed::Command {
                    command: path.clone(),
                    args: binary_args.unwrap_or_default(),
                    env: Default::default(),
                });
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let release = zed::latest_github_release(
            REPO,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();
        let asset_name = format!(
            "csharp-language-server-{arch}-{os}.{extension}",
            os = match platform {
                zed::Os::Mac => "apple-darwin",
                zed::Os::Linux => "unknown-linux-gnu",
                zed::Os::Windows => "pc-windows-msvc",
            },
            arch = match arch {
                zed::Architecture::Aarch64 => "aarch64",
                zed::Architecture::X8664 => "x86_64",
                zed::Architecture::X86 => "unsupported",
            },
            extension = match platform {
                zed::Os::Mac | zed::Os::Linux => "tar.gz",
                zed::Os::Windows => "zip",
            }
        );

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

        let version_dir = format!("roslyn-{}", release.version);
        let binary_path = format!("{version_dir}/csharp-language-server");

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &version_dir,
                match platform {
                    zed::Os::Mac | zed::Os::Linux => zed::DownloadedFileType::GzipTar,
                    zed::Os::Windows => zed::DownloadedFileType::Zip,
                },
            )
            .map_err(|e| format!("failed to download file: {e}"))?;

            zed::make_file_executable(&binary_path)?;

            let entries =
                fs::read_dir(".").map_err(|e| format!("failed to list working directory {e}"))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
                if entry.file_name().to_str() != Some(&version_dir) {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(zed::Command {
            command: binary_path,
            args: binary_args.unwrap_or_default(),
            env: Default::default(),
        })
    }

    pub fn configuration_options(
        &self,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        let settings = LspSettings::for_worktree("roslyn", worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings);

        if let Some(user_settings) = settings {
            let transformed_settings = self.transform_settings_for_roslyn(user_settings)?;
            return Ok(Some(transformed_settings));
        }

        Ok(None)
    }

    fn transform_settings_for_roslyn(
        &self,
        settings: zed::serde_json::Value,
    ) -> Result<zed::serde_json::Value> {
        let mut roslyn_config = Map::new();

        if let zed::serde_json::Value::Object(settings_map) = settings {
            for (key, value) in &settings_map {
                if key.contains('|') {
                    // This is already in the language|category format
                    if let zed::serde_json::Value::Object(nested_settings) = value {
                        for (nested_key, nested_value) in nested_settings {
                            // The key already contains the proper format, just add the setting
                            let roslyn_key = format!("{}.{}", key, nested_key);
                            roslyn_config.insert(roslyn_key, nested_value.clone());
                        }
                    }
                }
                // Handle direct roslyn-format settings (fallback for any other format)
                else if key.contains('.') {
                    roslyn_config.insert(key.clone(), value.clone());
                }
            }
        }

        Ok(zed::serde_json::Value::Object(roslyn_config))
    }
}
