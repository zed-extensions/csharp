use std::fs;

use zed_extension_api::{self as zed, settings::LspSettings, LanguageServerId, Result};

use crate::language_servers::{nuget::NuGetClient, util};

const PACKAGE_PREFIX: &str = "roslyn-language-server";
const SERVER_BINARY: &str = "Microsoft.CodeAnalysis.LanguageServer";

pub struct Roslyn {
    cached_server_path: Option<ServerPath>,
    nuget: NuGetClient,
}

impl Roslyn {
    pub const LANGUAGE_SERVER_ID: &'static str = "roslyn";

    pub fn new() -> Self {
        Roslyn {
            cached_server_path: None,
            nuget: NuGetClient::new(),
        }
    }

    pub fn language_server_cmd(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let binary_settings = LspSettings::for_worktree(Self::LANGUAGE_SERVER_ID, worktree)
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

        if let Some(ref server_path) = self.cached_server_path {
            if fs::metadata(server_path.as_str()).is_ok_and(|stat| stat.is_file()) {
                return Ok(Self::build_command(server_path, binary_args));
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let rid = match zed::current_platform() {
            (zed::Os::Windows, zed::Architecture::X8664) => "win-x64",
            (zed::Os::Windows, zed::Architecture::Aarch64) => "win-arm64",
            (zed::Os::Linux, zed::Architecture::X8664) => "linux-x64",
            (zed::Os::Linux, zed::Architecture::Aarch64) => "linux-arm64",
            (zed::Os::Mac, zed::Architecture::X8664) => "osx-x64",
            (zed::Os::Mac, zed::Architecture::Aarch64) => "osx-arm64",
            _ => "any",
        };

        let package_id = format!("{PACKAGE_PREFIX}.{rid}");
        let version = self.nuget.get_latest_version(&package_id)?;
        let version_dir = format!("{}-{}", Self::LANGUAGE_SERVER_ID, version);

        let already_installed = Self::find_server_path(rid, &version_dir)
            .is_ok_and(|sp| fs::metadata(sp.as_str()).is_ok_and(|stat| stat.is_file()));

        if !already_installed {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            self.nuget
                .download_and_extract(&package_id, &version, &version_dir)?;

            util::remove_outdated_versions(Self::LANGUAGE_SERVER_ID, &version_dir)?;
        }

        let server_path = Self::find_server_path(rid, &version_dir)?;
        if let ServerPath::Exe(ref path) = server_path {
            zed::make_file_executable(path)?;
        }

        let command = Self::build_command(&server_path, binary_args);
        self.cached_server_path = Some(server_path);
        Ok(command)
    }

    fn build_command(server_path: &ServerPath, user_args: Option<Vec<String>>) -> zed::Command {
        let mut extra_args = vec!["--stdio".to_string(), "--autoLoadProjects".to_string()];
        if let Some(args) = user_args {
            extra_args.extend(args);
        }

        match server_path {
            ServerPath::Dll(path) => {
                let mut args = vec!["exec".to_string(), path.clone()];
                args.extend(extra_args);
                zed::Command {
                    command: "dotnet".to_string(),
                    args,
                    env: Default::default(),
                }
            }
            ServerPath::Exe(path) => zed::Command {
                command: path.clone(),
                args: extra_args,
                env: Default::default(),
            },
        }
    }

    fn find_server_path(rid: &str, version_dir: &str) -> Result<ServerPath> {
        let tools_dir = format!("{version_dir}/tools");

        let tfm = fs::read_dir(&tools_dir)
            .map_err(|e| format!("failed to read tools directory '{tools_dir}': {e}"))?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                if entry.file_type().ok()?.is_dir() {
                    entry.file_name().into_string().ok()
                } else {
                    None
                }
            })
            .next()
            .ok_or_else(|| format!("no TFM directory found inside '{tools_dir}'"))?;

        let server_dir = format!("{tools_dir}/{tfm}/{rid}");
        Ok(Self::server_path_for_rid(rid, server_dir))
    }

    fn server_path_for_rid(rid: &str, server_dir: String) -> ServerPath {
        if rid == "any" {
            ServerPath::Dll(format!("{server_dir}/{SERVER_BINARY}.dll"))
        } else if rid.starts_with("win-") {
            ServerPath::Exe(format!("{server_dir}/{SERVER_BINARY}.exe"))
        } else {
            ServerPath::Exe(format!("{server_dir}/{SERVER_BINARY}"))
        }
    }

    pub fn configuration_options(
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        let settings = LspSettings::for_worktree(Self::LANGUAGE_SERVER_ID, worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings);

        Ok(settings.map(Self::transform_settings_for_roslyn))
    }

    fn transform_settings_for_roslyn(settings: zed::serde_json::Value) -> zed::serde_json::Value {
        let mut roslyn_config = zed::serde_json::json!({
            // These code lenses show up as "Unknown Command" in Zed and don't do anything when clicked. Disable them by default.
            "csharp|code_lens.dotnet_enable_references_code_lens": false,
            "csharp|code_lens.dotnet_enable_tests_code_lens": false,
            // Enable inlay hints in the language server by default.
            // This way, enabling inlay hints in Zed will cause inlay hints to show up in C# without extra configuration.
            "csharp|inlay_hints.dotnet_enable_inlay_hints_for_parameters": true,
            "csharp|inlay_hints.dotnet_enable_inlay_hints_for_literal_parameters": true,
            "csharp|inlay_hints.dotnet_enable_inlay_hints_for_indexer_parameters": true,
            "csharp|inlay_hints.dotnet_enable_inlay_hints_for_object_creation_parameters": true,
            "csharp|inlay_hints.dotnet_enable_inlay_hints_for_other_parameters": true,
            "csharp|inlay_hints.csharp_enable_inlay_hints_for_types": true,
            "csharp|inlay_hints.csharp_enable_inlay_hints_for_implicit_variable_types": true,
            "csharp|inlay_hints.csharp_enable_inlay_hints_for_lambda_parameter_types": true,
            "csharp|inlay_hints.csharp_enable_inlay_hints_for_implicit_object_creation": true,
            "csharp|inlay_hints.csharp_enable_inlay_hints_for_collection_expressions": true,
        });

        let config_map = roslyn_config.as_object_mut().unwrap();
        if let zed::serde_json::Value::Object(settings_map) = settings {
            for (key, value) in settings_map {
                if key.contains('|') {
                    // This is already in the language|category format
                    if let zed::serde_json::Value::Object(nested_settings) = value {
                        for (nested_key, nested_value) in nested_settings {
                            // The key already contains the proper format, just add the setting
                            config_map.insert(format!("{key}.{nested_key}"), nested_value);
                        }
                    }
                }
                // Handle direct roslyn-format settings (fallback for any other format)
                else if key.contains('.') {
                    config_map.insert(key.clone(), value.clone());
                }
            }
        }

        roslyn_config
    }
}

enum ServerPath {
    Exe(String),
    Dll(String),
}

impl ServerPath {
    fn as_str(&self) -> &str {
        match self {
            ServerPath::Exe(path) | ServerPath::Dll(path) => path,
        }
    }
}
