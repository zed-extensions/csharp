use std::fs;

use zed_extension_api::{self as zed, settings::LspSettings, LanguageServerId, Result};

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
            //Roslyn user config
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
        Err("Roslyn binary not found".to_string())
    }
}
