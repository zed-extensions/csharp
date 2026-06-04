use std::fs;

use zed_extension_api::{self as zed, settings::LspSettings, LanguageServerId, Result};

use crate::language_servers::{nuget::NuGetClient, util};

const PACKAGE_ID: &str = "csharp-ls";
const SERVER_DLL: &str = "CSharpLanguageServer.dll";
const DOTNET_HINT: &str = "csharp-ls requires the .NET SDK on PATH. Install .NET 10+ \
or set `lsp.csharp-ls.binary.path` to a working `csharp-ls` binary.";

pub struct CsharpLs {
    cached_dll_path: Option<String>,
    nuget: NuGetClient,
}

impl CsharpLs {
    pub const LANGUAGE_SERVER_ID: &'static str = "csharp-ls";
    const BINARY_NAME: &'static str = "csharp-ls";

    pub fn new() -> Self {
        Self {
            cached_dll_path: None,
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
        let binary_args = binary_settings.as_ref().and_then(|b| b.arguments.clone());

        if let Some(path) = binary_settings.and_then(|b| b.path) {
            return Ok(zed::Command {
                command: path,
                args: binary_args.unwrap_or_default(),
                env: Default::default(),
            });
        }

        if let Some(path) = worktree.which(Self::BINARY_NAME) {
            return Ok(zed::Command {
                command: path,
                args: binary_args.unwrap_or_default(),
                env: Default::default(),
            });
        }

        if let Some(ref dll_path) = self.cached_dll_path {
            if fs::metadata(dll_path).is_ok_and(|s| s.is_file()) {
                return Self::dotnet_exec(worktree, dll_path, binary_args);
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let version = self.nuget.get_latest_version(PACKAGE_ID)?;
        let version_dir = format!("{}-{}", Self::LANGUAGE_SERVER_ID, version);

        if Self::find_dll(&version_dir).is_err() {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            self.nuget
                .download_and_extract(PACKAGE_ID, &version, &version_dir)?;

            util::remove_outdated_versions(Self::LANGUAGE_SERVER_ID, &version_dir)?;
        }

        let dll_path = Self::find_dll(&version_dir)?;
        let command = Self::dotnet_exec(worktree, &dll_path, binary_args)?;
        self.cached_dll_path = Some(dll_path);
        Ok(command)
    }

    fn dotnet_exec(
        worktree: &zed::Worktree,
        dll_path: &str,
        user_args: Option<Vec<String>>,
    ) -> Result<zed::Command> {
        let dotnet = worktree
            .which("dotnet")
            .ok_or_else(|| DOTNET_HINT.to_string())?;
        let mut args = vec!["exec".to_string(), dll_path.to_string()];
        if let Some(user) = user_args {
            args.extend(user);
        }
        Ok(zed::Command {
            command: dotnet,
            args,
            env: Default::default(),
        })
    }

    fn find_dll(version_dir: &str) -> Result<String> {
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

        let dll_path = format!("{tools_dir}/{tfm}/any/{SERVER_DLL}");

        if fs::metadata(&dll_path).is_ok_and(|s| s.is_file()) {
            Ok(dll_path)
        } else {
            Err(format!(
                "csharp-ls package layout unexpected: missing entry DLL at '{dll_path}'"
            ))
        }
    }

    pub fn configuration_options(
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        let settings = LspSettings::for_worktree(Self::LANGUAGE_SERVER_ID, worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings);
        Ok(settings.map(|s| zed::serde_json::json!({ "csharp": s })))
    }
}
