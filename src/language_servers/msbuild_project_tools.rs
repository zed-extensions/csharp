use std::fs;

use zed_extension_api::{self as zed, settings::LspSettings, LanguageServerId, Result};

use crate::language_servers::util;

const GITHUB_REPO: &str = "tintoy/msbuild-project-tools-server";
const RELEASE_ASSET: &str = "language-server.zip";
const SERVER_DLL: &str = "MSBuildProjectTools.LanguageServer.Host.dll";
const SETTINGS_SECTION: &str = "msbuildProjectTools";
const DOTNET_HINT: &str = "MSBuild Project Tools requires the .NET SDK on PATH. Install the .NET \
SDK or set `lsp.msbuild-project-tools.binary.path` to a working language server binary.";

pub struct MsbuildProjectTools {
    cached_dll_path: Option<String>,
}

impl MsbuildProjectTools {
    pub const LANGUAGE_SERVER_ID: &'static str = "msbuild-project-tools";

    pub fn new() -> Self {
        Self {
            cached_dll_path: None,
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
                env: Self::env(worktree),
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

        let release = zed::latest_github_release(
            GITHUB_REPO,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == RELEASE_ASSET)
            .ok_or_else(|| format!("no asset found matching {RELEASE_ASSET:?}"))?;

        let version_dir = format!("{}-{}", Self::LANGUAGE_SERVER_ID, release.version);
        let dll_path = format!("{version_dir}/{SERVER_DLL}");

        if !fs::metadata(&dll_path).is_ok_and(|s| s.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &version_dir,
                zed::DownloadedFileType::Zip,
            )
            .map_err(|e| format!("failed to download file: {e}"))?;

            util::remove_outdated_versions(Self::LANGUAGE_SERVER_ID, &version_dir)?;

            if !fs::metadata(&dll_path).is_ok_and(|s| s.is_file()) {
                return Err(format!(
                    "msbuild-project-tools package layout unexpected: missing entry DLL at '{dll_path}'"
                ));
            }
        }

        let dll_path = util::absolute_path(&dll_path)?;
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
            env: Self::env(worktree),
        })
    }

    fn env(worktree: &zed::Worktree) -> Vec<(String, String)> {
        let mut env = vec![
            ("DOTNET_ROLL_FORWARD".to_string(), "LatestMajor".to_string()),
            (
                "DOTNET_ROLL_FORWARD_TO_PRERELEASE".to_string(),
                "1".to_string(),
            ),
        ];

        let Some(settings) = Self::user_settings(worktree) else {
            return env;
        };
        let logging = &settings["logging"];

        if logging["level"] == "Verbose" {
            env.push((
                "MSBUILD_PROJECT_TOOLS_VERBOSE_LOGGING".to_string(),
                "1".to_string(),
            ));
        }
        if let Some(log_file) = logging["file"].as_str() {
            env.push((
                "MSBUILD_PROJECT_TOOLS_LOG_FILE".to_string(),
                log_file.to_string(),
            ));
        }
        if let Some(seq_url) = logging["seq"]["url"].as_str() {
            env.push((
                "MSBUILD_PROJECT_TOOLS_SEQ_URL".to_string(),
                seq_url.to_string(),
            ));

            if let Some(api_key) = logging["seq"]["apiKey"].as_str() {
                env.push((
                    "MSBUILD_PROJECT_TOOLS_SEQ_API_KEY".to_string(),
                    api_key.to_string(),
                ));
            }
        }

        env
    }

    fn user_settings(worktree: &zed::Worktree) -> Option<zed::serde_json::Value> {
        LspSettings::for_worktree(Self::LANGUAGE_SERVER_ID, worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings)
    }

    pub fn initialization_options(
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        Self::configuration_options(worktree)
    }

    pub fn configuration_options(
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        Ok(Self::user_settings(worktree).map(|s| zed::serde_json::json!({ SETTINGS_SECTION: s })))
    }
}
