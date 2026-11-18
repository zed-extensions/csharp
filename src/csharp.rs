mod language_servers;

use std::collections::HashMap;

use zed_extension_api::{self as zed, settings::LspSettings, Result};

use crate::language_servers::{CsharpLs, Omnisharp, Roslyn};

struct CsharpExtension {
    omnisharp: Option<Omnisharp>,
    roslyn: Option<Roslyn>,
    csharp_ls: Option<CsharpLs>,
}

impl CsharpExtension {}

fn merge_environment(
    mut inherited: zed::EnvVars,
    overrides: HashMap<String, String>,
    windows: bool,
) -> Result<zed::EnvVars> {
    for (name, value) in overrides {
        if name.is_empty() || name.contains(['=', '\0']) || value.contains('\0') {
            return Err(
                "Invalid LSP binary.env: names must be nonempty and contain neither '=' nor NUL; values must not contain NUL."
                    .to_string(),
            );
        }
        inherited.retain(|(existing, _)| {
            if windows {
                !existing.eq_ignore_ascii_case(&name)
            } else {
                existing != &name
            }
        });
        inherited.push((name, value));
    }
    Ok(inherited)
}

impl zed::Extension for CsharpExtension {
    fn new() -> Self {
        Self {
            omnisharp: None,
            roslyn: None,
            csharp_ls: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let overrides = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?
            .binary
            .and_then(|binary| binary.env)
            .unwrap_or_default();
        let env = merge_environment(
            worktree.shell_env(),
            overrides,
            matches!(zed::current_platform().0, zed::Os::Windows),
        )?;
        let mut command = match language_server_id.as_ref() {
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
            Roslyn::LANGUAGE_SERVER_ID => {
                // Add Roslyn Server
                let roslyn = self.roslyn.get_or_insert_with(Roslyn::new);
                roslyn.language_server_cmd(language_server_id, worktree)
            }
            CsharpLs::LANGUAGE_SERVER_ID => {
                let csharp_ls = self.csharp_ls.get_or_insert_with(CsharpLs::new);
                csharp_ls.language_server_cmd(language_server_id, worktree)
            }
            language_server_id => Err(format!("unknown language server: {language_server_id}")),
        }?;
        // MSBuild and NuGet must see the same SDK and package-cache settings as the worktree.
        command.env = env;
        Ok(command)
    }

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        match language_server_id.as_ref() {
            Roslyn::LANGUAGE_SERVER_ID => Roslyn::configuration_options(worktree),
            CsharpLs::LANGUAGE_SERVER_ID => CsharpLs::configuration_options(worktree),
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_environment_overrides_shell_and_preserves_other_variables() {
        let env = merge_environment(
            vec![
                ("PATH".into(), "tools".into()),
                ("NUGET_PACKAGES".into(), "old-cache".into()),
            ],
            HashMap::from([
                ("NUGET_PACKAGES".into(), "workspace-cache".into()),
                ("CUSTOM_SETTING".into(), "enabled".into()),
                ("EMPTY_SETTING".into(), "".into()),
            ]),
            false,
        )
        .unwrap();
        assert_eq!(env.len(), 4);
        let env: HashMap<_, _> = env.into_iter().collect();
        assert_eq!(env["PATH"], "tools");
        assert_eq!(env["NUGET_PACKAGES"], "workspace-cache");
        assert_eq!(env["CUSTOM_SETTING"], "enabled");
        assert_eq!(env["EMPTY_SETTING"], "");
    }

    #[test]
    fn windows_environment_overrides_are_case_insensitive() {
        let env = merge_environment(
            vec![("Path".into(), "old-path".into())],
            HashMap::from([("PATH".into(), "new-path".into())]),
            true,
        )
        .unwrap();
        assert_eq!(env, vec![("PATH".to_string(), "new-path".to_string())]);
    }

    #[test]
    fn unix_environment_preserves_distinct_case() {
        let env = merge_environment(
            vec![("Path".into(), "old-path".into())],
            HashMap::from([("PATH".into(), "new-path".into())]),
            false,
        )
        .unwrap();
        assert_eq!(env.len(), 2);
    }

    #[test]
    fn rejects_invalid_environment_without_exposing_values() {
        for (name, value) in [
            ("", "secret"),
            ("INVALID=NAME", "secret"),
            ("INVALID\0NAME", "secret"),
            ("NAME", "secret\0value"),
        ] {
            let error =
                merge_environment(vec![], HashMap::from([(name.into(), value.into())]), false)
                    .unwrap_err();
            assert!(!error.contains("secret"));
        }
    }
}

zed::register_extension!(CsharpExtension);
