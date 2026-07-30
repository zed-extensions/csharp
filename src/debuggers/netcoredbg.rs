//! Debug adapter support, backed by [netcoredbg].
//!
//! netcoredbg speaks DAP when started with `--interpreter=vscode`, which is
//! what Zed's debugger expects. Releases are published as per-platform
//! archives on GitHub, so the adapter is downloaded on first use in the same
//! way the language servers are.
//!
//! [netcoredbg]: https://github.com/Samsung/netcoredbg

use std::fs;

use zed_extension_api::{
    self as zed, serde_json,
    serde_json::{json, Value},
    DebugAdapterBinary, DebugConfig, DebugRequest, DebugScenario, DebugTaskDefinition,
    LaunchRequest, Result, StartDebuggingRequestArguments, StartDebuggingRequestArgumentsRequest,
    TaskTemplate,
};

use crate::language_servers::util;

const REPOSITORY: &str = "Samsung/netcoredbg";
/// Prefix of the directory each downloaded release is extracted into.
const INSTALL_PREFIX: &str = "netcoredbg";

pub struct Netcoredbg {
    /// Path to an adapter binary that was already resolved this session.
    cached_binary_path: Option<String>,
}

impl Netcoredbg {
    pub const ADAPTER_NAME: &'static str = "netcoredbg";

    pub fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    pub fn dap_binary(
        &mut self,
        definition: DebugTaskDefinition,
        user_provided_path: Option<String>,
    ) -> Result<DebugAdapterBinary> {
        let configuration: Value = serde_json::from_str(&definition.config)
            .map_err(|err| format!("invalid debug configuration: {err}"))?;

        let command = match user_provided_path {
            Some(path) => path,
            None => self.resolve_binary_path()?,
        };

        let cwd = configuration
            .get("cwd")
            .and_then(Value::as_str)
            .map(str::to_owned);

        Ok(DebugAdapterBinary {
            command: Some(command),
            arguments: vec!["--interpreter=vscode".into()],
            envs: Vec::new(),
            cwd,
            connection: None,
            request_args: StartDebuggingRequestArguments {
                request: request_kind(&configuration)?,
                configuration: definition.config,
            },
        })
    }

    /// Returns the path to a usable adapter binary, downloading one if needed.
    fn resolve_binary_path(&mut self) -> Result<String> {
        if let Some(path) = self.cached_binary_path.as_ref() {
            if fs::metadata(path).is_ok_and(|stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        let release = zed::latest_github_release(
            REPOSITORY,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let asset_name = asset_name()?;
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| {
                format!(
                    "netcoredbg {} does not ship a '{asset_name}' build; \
                     set `dap.netcoredbg.binary` to an adapter you built yourself",
                    release.version
                )
            })?;

        let install_dir = format!("{INSTALL_PREFIX}-{}", release.version);
        // Both archive kinds extract to a `netcoredbg/` directory.
        let binary_path = format!("{install_dir}/netcoredbg/{}", binary_name());

        if !fs::metadata(&binary_path).is_ok_and(|stat| stat.is_file()) {
            let file_type = if asset_name.ends_with(".zip") {
                zed::DownloadedFileType::Zip
            } else {
                zed::DownloadedFileType::GzipTar
            };

            zed::download_file(&asset.download_url, &install_dir, file_type)
                .map_err(|err| format!("failed to download netcoredbg: {err}"))?;

            util::remove_outdated_versions(INSTALL_PREFIX, &install_dir)?;
        }

        zed::make_file_executable(&binary_path)?;

        let binary_path = util::absolute_path(&binary_path)?;
        self.cached_binary_path = Some(binary_path.clone());

        Ok(binary_path)
    }

    /// Translates Zed's adapter-agnostic debug configuration into netcoredbg's
    /// own launch/attach schema.
    pub fn config_to_scenario(config: DebugConfig) -> Result<DebugScenario> {
        let configuration = match config.request {
            DebugRequest::Launch(launch) => json!({
                "request": "launch",
                "program": launch.program,
                "args": launch.args,
                "cwd": launch.cwd,
                "env": launch
                    .envs
                    .into_iter()
                    .map(|(key, value)| (key, Value::String(value)))
                    .collect::<serde_json::Map<String, Value>>(),
                "stopAtEntry": config.stop_on_entry.unwrap_or(false),
                "justMyCode": true,
            }),
            DebugRequest::Attach(attach) => json!({
                "request": "attach",
                "processId": attach.process_id,
            }),
        };

        Ok(DebugScenario {
            label: config.label,
            adapter: config.adapter,
            build: None,
            config: configuration.to_string(),
            tcp_connection: None,
        })
    }
}

/// Reads the `request` field of a debug configuration.
pub fn request_kind(configuration: &Value) -> Result<StartDebuggingRequestArgumentsRequest> {
    match configuration.get("request").and_then(Value::as_str) {
        Some("launch") => Ok(StartDebuggingRequestArgumentsRequest::Launch),
        Some("attach") => Ok(StartDebuggingRequestArgumentsRequest::Attach),
        Some(other) => Err(format!(
            "unsupported debug request '{other}'; expected \"launch\" or \"attach\""
        )),
        None => Err("debug configuration is missing a \"request\" field".into()),
    }
}

/// Name of the release asset for the current platform.
fn asset_name() -> Result<String> {
    let name = match zed::current_platform() {
        (zed::Os::Mac, zed::Architecture::Aarch64) => "netcoredbg-osx-arm64.zip",
        (zed::Os::Mac, zed::Architecture::X8664) => "netcoredbg-osx-amd64.zip",
        (zed::Os::Linux, zed::Architecture::Aarch64) => "netcoredbg-linux-arm64.tar.gz",
        (zed::Os::Linux, zed::Architecture::X8664) => "netcoredbg-linux-amd64.tar.gz",
        (zed::Os::Windows, _) => "netcoredbg-win64.zip",
        (os, architecture) => {
            return Err(format!(
                "netcoredbg does not publish a build for {os:?} {architecture:?}"
            ))
        }
    };

    Ok(name.to_owned())
}

fn binary_name() -> &'static str {
    match zed::current_platform().0 {
        zed::Os::Windows => "netcoredbg.exe",
        _ => "netcoredbg",
    }
}

/// The `dotnet` debug locator.
///
/// A `dotnet run` task does not name the assembly Zed needs to hand to the
/// debugger, so the task is rewritten as a `dotnet build` and the assembly path
/// is recovered afterwards by asking MSBuild for `TargetPath`.
pub mod locator {
    use super::*;
    use zed_extension_api::{BuildTaskDefinition, BuildTaskDefinitionTemplatePayload};

    pub const NAME: &str = "dotnet";

    /// Verbs whose `TargetPath` is an assembly that can be launched directly.
    ///
    /// `test` is deliberately absent. MSBuild happily reports a `TargetPath`
    /// for a test project, but that assembly is a library driven by VSTest —
    /// handing it to the debugger produces a session that exits immediately
    /// without running a single test. Debugging an individual test needs the
    /// test host to be started under the debugger instead (see the
    /// `VSTEST_HOST_DEBUG` task), which a locator cannot arrange. `watch` is
    /// absent for the same reason: the process the debugger would need is a
    /// child of `dotnet watch`, not the assembly MSBuild names.
    const DEBUGGABLE_VERBS: &[&str] = &["run"];

    pub fn create_scenario(
        build_task: TaskTemplate,
        resolved_label: String,
        adapter_name: String,
    ) -> Option<DebugScenario> {
        if build_task.command != "dotnet" {
            return None;
        }

        let verb = build_task.args.first()?;
        if !DEBUGGABLE_VERBS.contains(&verb.as_str()) {
            return None;
        }

        Some(DebugScenario {
            label: resolved_label,
            adapter: adapter_name,
            build: Some(BuildTaskDefinition::Template(
                BuildTaskDefinitionTemplatePayload {
                    locator_name: Some(NAME.to_owned()),
                    template: to_build_task(build_task),
                },
            )),
            config: String::new(),
            tcp_connection: None,
        })
    }

    pub fn run(build_task: TaskTemplate) -> Result<DebugRequest> {
        let target = msbuild_target(&build_task)
            .ok_or_else(|| "could not determine which project to debug".to_string())?;

        let output = zed::process::Command::new("dotnet")
            .args([
                "msbuild",
                target.as_str(),
                "-getProperty:TargetPath",
                "-nologo",
                "-verbosity:quiet",
            ])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();

        if output.status != Some(0) {
            // MSBuild reports failures such as MSB1003 on stdout and leaves
            // stderr empty, so both streams have to be considered to produce a
            // message the user can act on.
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            let reason = [stderr, stdout]
                .into_iter()
                .find(|stream| !stream.is_empty())
                .unwrap_or_else(|| "no output".to_string());

            return Err(format!(
                "`dotnet msbuild {target} -getProperty:TargetPath` failed: {reason}"
            ));
        }

        if stdout.is_empty() {
            return Err(format!("MSBuild reported no TargetPath for '{target}'"));
        }

        Ok(DebugRequest::Launch(LaunchRequest {
            program: stdout,
            cwd: build_task.cwd,
            args: Vec::new(),
            envs: build_task.env,
        }))
    }

    /// Rewrites `dotnet <verb> ...` as `dotnet build ...`, dropping the
    /// arguments that only make sense for the original verb.
    fn to_build_task(mut template: TaskTemplate) -> TaskTemplate {
        let mut args = vec!["build".to_owned()];

        // Everything after `--` is passed to the program being run, not to the
        // SDK, so it has no meaning for a build.
        let sdk_args = template
            .args
            .iter()
            .skip(1)
            .take_while(|arg| arg.as_str() != "--");

        args.extend(sdk_args.cloned());
        template.args = args;
        template
    }

    /// Picks the project or solution to query MSBuild about.
    fn msbuild_target(build_task: &TaskTemplate) -> Option<String> {
        let explicit = build_task
            .args
            .iter()
            .find(|arg| is_project_file(arg))
            .cloned();

        explicit.or_else(|| build_task.cwd.clone())
    }

    fn is_project_file(arg: &str) -> bool {
        [".csproj", ".fsproj", ".vbproj"]
            .iter()
            .any(|suffix| arg.ends_with(suffix))
    }
}
