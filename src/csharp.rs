mod debuggers;
mod language_servers;

use zed_extension_api::{
    self as zed, DebugAdapterBinary, DebugConfig, DebugRequest, DebugScenario, DebugTaskDefinition,
    Result, StartDebuggingRequestArgumentsRequest, TaskTemplate,
};

use crate::debuggers::{netcoredbg, Netcoredbg};
use crate::language_servers::{CsharpLs, Omnisharp, Roslyn};

struct CsharpExtension {
    omnisharp: Option<Omnisharp>,
    roslyn: Option<Roslyn>,
    csharp_ls: Option<CsharpLs>,
    netcoredbg: Option<Netcoredbg>,
}

impl CsharpExtension {}

impl zed::Extension for CsharpExtension {
    fn new() -> Self {
        Self {
            omnisharp: None,
            roslyn: None,
            csharp_ls: None,
            netcoredbg: None,
        }
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
        }
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

    fn get_dap_binary(
        &mut self,
        adapter_name: String,
        config: DebugTaskDefinition,
        user_provided_debug_adapter_path: Option<String>,
        _worktree: &zed::Worktree,
    ) -> Result<DebugAdapterBinary> {
        match adapter_name.as_str() {
            Netcoredbg::ADAPTER_NAME => self
                .netcoredbg
                .get_or_insert_with(Netcoredbg::new)
                .dap_binary(config, user_provided_debug_adapter_path),
            adapter_name => Err(format!("unknown debug adapter: {adapter_name}")),
        }
    }

    fn dap_request_kind(
        &mut self,
        adapter_name: String,
        config: zed::serde_json::Value,
    ) -> Result<StartDebuggingRequestArgumentsRequest> {
        match adapter_name.as_str() {
            Netcoredbg::ADAPTER_NAME => netcoredbg::request_kind(&config),
            adapter_name => Err(format!("unknown debug adapter: {adapter_name}")),
        }
    }

    fn dap_config_to_scenario(&mut self, config: DebugConfig) -> Result<DebugScenario> {
        match config.adapter.as_str() {
            Netcoredbg::ADAPTER_NAME => Netcoredbg::config_to_scenario(config),
            adapter_name => Err(format!("unknown debug adapter: {adapter_name}")),
        }
    }

    fn dap_locator_create_scenario(
        &mut self,
        locator_name: String,
        build_task: TaskTemplate,
        resolved_label: String,
        debug_adapter_name: String,
    ) -> Option<DebugScenario> {
        if locator_name != netcoredbg::locator::NAME {
            return None;
        }

        netcoredbg::locator::create_scenario(build_task, resolved_label, debug_adapter_name)
    }

    fn run_dap_locator(
        &mut self,
        locator_name: String,
        build_task: TaskTemplate,
    ) -> Result<DebugRequest> {
        match locator_name.as_str() {
            netcoredbg::locator::NAME => netcoredbg::locator::run(build_task),
            locator_name => Err(format!("unknown debug locator: {locator_name}")),
        }
    }
}

zed::register_extension!(CsharpExtension);
