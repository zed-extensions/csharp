mod binary_manager;
mod language_servers;
mod simple_temp_dir;

use binary_manager::BinaryManager;
use language_servers::Roslyn;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::Ipv4Addr};
use zed_extension_api::{
    self as zed, resolve_tcp_template, DebugAdapterBinary, DebugConfig, DebugRequest,
    DebugScenario, DebugTaskDefinition, Result, StartDebuggingRequestArguments,
    StartDebuggingRequestArgumentsRequest, TcpArgumentsTemplate, Worktree,
};

use crate::language_servers::Omnisharp;

struct CsharpExtension {
    omnisharp: Option<Omnisharp>,
    roslyn: Option<Roslyn>,
    binary_manager: BinaryManager,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NetCoreDbgDebugConfig {
    pub request: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_at_entry: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_id: Option<ProcessId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub just_my_code: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_step_filtering: Option<bool>,
}

/// Represents a process id that can be either an integer or a string (containing a number)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum ProcessId {
    Int(i32),
    String(String),
}

impl CsharpExtension {
    const ADAPTER_NAME: &str = "netcoredbg";
    const LOCATOR_NAME: &str = "csharp-test-runner";
}

impl zed::Extension for CsharpExtension {
    fn new() -> Self {
        Self {
            omnisharp: None,
            roslyn: None,
            binary_manager: BinaryManager::new(),
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
            language_server_id => Err(format!("unknown language server: {language_server_id}")),
        }
    }

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        if language_server_id.as_ref() == Roslyn::LANGUAGE_SERVER_ID {
            return Roslyn::configuration_options(worktree);
        }
        Ok(None)
    }

    fn get_dap_binary(
        &mut self,
        adapter_name: String,
        debug_task_definition: DebugTaskDefinition,
        user_provided_debug_adapter_path: Option<String>,
        worktree: &Worktree,
    ) -> Result<DebugAdapterBinary, String> {
        if adapter_name != Self::ADAPTER_NAME {
            return Err(format!("Cannot create binary for adapter: {adapter_name}"));
        }

        let configuration = debug_task_definition.config.to_string();
        let parsed_config: NetCoreDbgDebugConfig =
            zed::serde_json::from_str(&configuration).map_err(|e| {
                format!(
                    "Failed to parse debug configuration: {}. Expected NetCoreDbg configuration format.",
                    e
                )
            })?;

        let request = match parsed_config.request.as_str() {
            "launch" => StartDebuggingRequestArgumentsRequest::Launch,
            "attach" => StartDebuggingRequestArgumentsRequest::Attach,
            other => {
                return Err(format!(
                    "Invalid 'request' value: '{}'. Expected 'launch' or 'attach'",
                    other
                ))
            }
        };

        let tcp_connection = debug_task_definition
            .tcp_connection
            .unwrap_or(TcpArgumentsTemplate {
                port: None,
                host: None,
                timeout: None,
            });

        if request == StartDebuggingRequestArgumentsRequest::Attach {
            if parsed_config.process_id.is_none() {
                return Err("Attach request missing 'processId'".to_string());
            }
        }

        let binary_path = self
            .binary_manager
            .get_binary_path(user_provided_debug_adapter_path)?;

        let mut envs = parsed_config.env.clone();
        for (key, value) in worktree.shell_env() {
            envs.insert(key, value);
        }

        let mut args: Vec<String> = vec!["--interpreter=vscode".into()];
        if tcp_connection.host.is_some() {
            args.push("--server".into());
        }

        let connection = resolve_tcp_template(tcp_connection)?;

        Ok(DebugAdapterBinary {
            command: Some(binary_path),
            arguments: args,
            envs: envs.into_iter().collect(),
            cwd: Some(parsed_config.cwd.unwrap_or_else(|| worktree.root_path())),
            connection: Some(connection),
            request_args: StartDebuggingRequestArguments {
                configuration,
                request,
            },
        })
    }

    fn dap_request_kind(
        &mut self,
        adapter_name: String,
        config: zed::serde_json::Value,
    ) -> Result<StartDebuggingRequestArgumentsRequest, String> {
        if adapter_name != Self::ADAPTER_NAME {
            return Err(format!("Unknown adapter: {}", adapter_name));
        }

        match config.get("request").and_then(|v| v.as_str()) {
            Some("launch") => Ok(StartDebuggingRequestArgumentsRequest::Launch),
            Some("attach") => {
                if config.get("processId").is_none()
                    || config.get("processId").is_some_and(|v| v.is_null())
                {
                    return Err("Attach request missing 'processId'".to_string());
                }
                Ok(StartDebuggingRequestArgumentsRequest::Attach)
            }
            Some(other) => Err(format!(
                "Invalid 'request' value: '{}'. Expected 'launch' or 'attach'",
                other
            )),
            None => Err("Missing 'request' field. Expected 'launch' or 'attach'".to_string()),
        }
    }

    fn dap_config_to_scenario(&mut self, config: DebugConfig) -> Result<DebugScenario, String> {
        match config.request {
            DebugRequest::Launch(launch) => {
                let adapter_config = NetCoreDbgDebugConfig {
                    request: "launch".to_string(),
                    program: Some(launch.program),
                    args: if launch.args.is_empty() {
                        None
                    } else {
                        Some(launch.args)
                    },
                    cwd: launch.cwd,
                    env: launch.envs.into_iter().collect(),
                    stop_at_entry: config.stop_on_entry,
                    process_id: None,
                    just_my_code: Some(false),
                    enable_step_filtering: Some(true),
                };

                let config_json = zed::serde_json::to_string(&adapter_config)
                    .map_err(|e| format!("Failed to serialize launch config: {}", e))?;
                Ok(DebugScenario {
                    label: config.label,
                    adapter: config.adapter,
                    build: None,
                    config: config_json,
                    tcp_connection: None,
                })
            }
            DebugRequest::Attach(attach) => {
                let process_id = attach.process_id.ok_or_else(|| {
                    "Attach mode requires a process ID. Please select a process from the attach modal.".to_string()
                })?;

                let adapter_config = NetCoreDbgDebugConfig {
                    request: "attach".to_string(),
                    program: None,
                    args: None,
                    cwd: None,
                    env: HashMap::new(),
                    stop_at_entry: config.stop_on_entry,
                    process_id: Some(ProcessId::Int(process_id as i32)),
                    just_my_code: Some(false),
                    enable_step_filtering: Some(true),
                };

                let config_json = zed::serde_json::to_string(&adapter_config)
                    .map_err(|e| format!("Failed to serialize attach config: {}", e))?;

                Ok(DebugScenario {
                    label: config.label,
                    adapter: config.adapter,
                    build: None,
                    config: config_json,
                    tcp_connection: None,
                })
            }
        }
    }

    fn dap_locator_create_scenario(
        &mut self,
        locator_name: String,
        build_task: zed::TaskTemplate,
        resolved_label: String,
        debug_adapter_name: String,
    ) -> Option<DebugScenario> {
        if debug_adapter_name != Self::ADAPTER_NAME || locator_name != Self::LOCATOR_NAME {
            return None;
        }

        let mut args_iter = build_task.args.iter();
        let subcommand = args_iter.next()?;
        if subcommand != "test" {
            return None;
        }

        if build_task.command != "dotnet" && !build_task.command.starts_with("$ZED_") {
            return None;
        }

        let file_dir = env_value(&build_task.env, "CSHARP_TEST_FILE_DIR")
            .or_else(|| build_task.cwd.clone())
            .unwrap_or_else(|| "ZED_DIRNAME".to_string());

        let mut env: Vec<(String, String)> = build_task.env.into();

        // TODO: Currently we halt the test process to wait for attach.
        // Zed extension API does not currently allow parallel process or process inspection to find what the build template PID is
        env.push(("VSTEST_HOST_DEBUG".to_string(), "1".to_string()));

        let args = vec!["test".into()];

        let template = zed::BuildTaskTemplate {
            label: "dotnet test (debug)".into(),
            command: "dotnet".into(),
            cwd: Some(file_dir),
            args,
            env,
        };

        let build_template =
            zed::BuildTaskDefinition::Template(zed::BuildTaskDefinitionTemplatePayload {
                template,
                locator_name: Some(locator_name),
            });

        Some(DebugScenario {
            adapter: debug_adapter_name,
            label: resolved_label,
            build: Some(build_template),
            // No config, this will trigger `run_dap_locator` so we can do the build step first
            config: "{}".into(),
            tcp_connection: None,
        })
    }

    fn run_dap_locator(
        &mut self,
        locator_name: String,
        _build_task: zed::TaskTemplate,
    ) -> Result<DebugRequest, String> {
        if locator_name != Self::LOCATOR_NAME {
            return Err(format!("Unknown locator: {locator_name}"));
        }

        // TODO: Currently this fails, but the user can read from debug outputs for the waiting build PID
        Ok(DebugRequest::Attach(zed::AttachRequest {
            process_id: None,
        }))
    }
}

zed::register_extension!(CsharpExtension);

fn env_value(env: &Vec<(String, String)>, key: &str) -> Option<String> {
    env.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
}
