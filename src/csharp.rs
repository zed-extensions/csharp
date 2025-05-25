mod language_servers;

use zed_extension_api::{self as zed, Result};

use crate::language_servers::Omnisharp;

struct CsharpExtension {
    omnisharp: Option<Omnisharp>,
}

impl CsharpExtension {}

impl zed::Extension for CsharpExtension {
    fn new() -> Self {
        Self { omnisharp: None }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        match language_server_id.as_ref() {
            Omnisharp::LANGUAGE_SERVER_ID => {
                let omnisharp = self.omnisharp.get_or_insert_with(Omnisharp::new);
                let omnisharp_binary =
                    omnisharp.language_server_binary(language_server_id, worktree)?;
                Ok(zed::Command {
                    command: omnisharp_binary.path,
                    args: omnisharp_binary.args.unwrap_or_else(|| vec!["-lsp".into()]),
                    env: Default::default(),
                })
            }
            language_server_id => Err(format!("unknown language server: {language_server_id}")),
        }
    }
}

zed::register_extension!(CsharpExtension);
