//! Debugging via [netcoredbg](https://github.com/Samsung/netcoredbg), plus a
//! `dotnet` locator that turns a `dotnet run` task into a debug session.

use std::fs;

use zed_extension_api::{
    self as zed,
    serde_json::{self, json, Map, Value},
    BuildTaskDefinition, BuildTaskDefinitionTemplatePayload, DebugAdapterBinary, DebugConfig,
    DebugRequest, DebugScenario, DebugTaskDefinition, LaunchRequest, Result,
    StartDebuggingRequestArguments, StartDebuggingRequestArgumentsRequest, TaskTemplate,
};

use crate::language_servers::util;

pub const LOCATOR_NAME: &str = "dotnet";

#[derive(Default)]
pub struct Netcoredbg {
    cached_binary_path: Option<String>,
}

impl Netcoredbg {
    pub const ADAPTER_NAME: &'static str = "netcoredbg";

    pub fn dap_binary(
        &mut self,
        task: DebugTaskDefinition,
        user_provided_path: Option<String>,
    ) -> Result<DebugAdapterBinary> {
        let config: Value = serde_json::from_str(&task.config)
            .map_err(|err| format!("invalid debug configuration: {err}"))?;
        let command = match user_provided_path {
            Some(path) => path,
            None => self.binary_path()?,
        };

        Ok(DebugAdapterBinary {
            command: Some(command),
            arguments: vec!["--interpreter=vscode".into()],
            envs: Vec::new(),
            cwd: config.get("cwd").and_then(Value::as_str).map(Into::into),
            connection: None,
            request_args: StartDebuggingRequestArguments {
                request: request_kind(&config)?,
                configuration: task.config,
            },
        })
    }

    fn binary_path(&mut self) -> Result<String> {
        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).is_ok_and(|stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        let (os, arch) = zed::current_platform();
        let asset_name = match (os, arch) {
            (zed::Os::Mac, zed::Architecture::Aarch64) => "netcoredbg-osx-arm64.zip",
            (zed::Os::Linux, zed::Architecture::Aarch64) => "netcoredbg-linux-arm64.tar.gz",
            (zed::Os::Linux, zed::Architecture::X8664) => "netcoredbg-linux-amd64.tar.gz",
            (zed::Os::Windows, _) => "netcoredbg-win64.zip",
            (os, arch) => {
                return Err(format!(
                    "netcoredbg has no build for {os:?} {arch:?}; set `dap.netcoredbg.binary`"
                ))
            }
        };

        let release = zed::latest_github_release(
            "Samsung/netcoredbg",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| {
                format!(
                    "netcoredbg {} has no asset '{asset_name}'; set `dap.netcoredbg.binary`",
                    release.version
                )
            })?;

        let version_dir = format!("netcoredbg-{}", release.version);
        let exe = match os {
            zed::Os::Windows => "netcoredbg.exe",
            _ => "netcoredbg",
        };
        // Every release archive extracts to a `netcoredbg/` directory.
        let binary_path = format!("{version_dir}/netcoredbg/{exe}");

        if !fs::metadata(&binary_path).is_ok_and(|stat| stat.is_file()) {
            let file_type = if asset_name.ends_with(".zip") {
                zed::DownloadedFileType::Zip
            } else {
                zed::DownloadedFileType::GzipTar
            };
            zed::download_file(&asset.download_url, &version_dir, file_type)
                .map_err(|err| format!("failed to download netcoredbg: {err}"))?;
            util::remove_outdated_versions("netcoredbg", &version_dir)?;
        }
        zed::make_file_executable(&binary_path)?;

        let binary_path = util::absolute_path(&binary_path)?;
        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

pub fn request_kind(config: &Value) -> Result<StartDebuggingRequestArgumentsRequest> {
    match config.get("request").and_then(Value::as_str) {
        Some("launch") => Ok(StartDebuggingRequestArgumentsRequest::Launch),
        Some("attach") => Ok(StartDebuggingRequestArgumentsRequest::Attach),
        Some(other) => Err(format!(
            "unsupported request '{other}'; expected launch or attach"
        )),
        None => Err("debug configuration is missing \"request\"".into()),
    }
}

pub fn config_to_scenario(config: DebugConfig) -> Result<DebugScenario> {
    let adapter_config = match config.request {
        DebugRequest::Launch(launch) => json!({
            "request": "launch",
            "program": launch.program,
            "args": launch.args,
            "cwd": launch.cwd,
            "env": launch.envs.into_iter().map(|(k, v)| (k, Value::String(v))).collect::<Map<_, _>>(),
            "stopAtEntry": config.stop_on_entry.unwrap_or(false),
            "justMyCode": true,
        }),
        DebugRequest::Attach(attach) => json!({
            "request": "attach",
            "processId": attach.process_id.ok_or("attaching requires a process id")?,
        }),
    };

    Ok(DebugScenario {
        label: config.label,
        adapter: config.adapter,
        build: None,
        config: adapter_config.to_string(),
        tcp_connection: None,
    })
}

/// Turns `dotnet run [options] [-- program args]` into a `dotnet build` task.
pub fn locator_scenario(
    task: TaskTemplate,
    label: String,
    adapter: String,
) -> Option<DebugScenario> {
    if task.command != "dotnet" || task.args.first()? != "run" {
        return None;
    }

    let mut args = task.args[1..].iter().cloned();
    let mut project = None;
    let mut build_args = Vec::new();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--" => break,
            // `dotnet build` takes the project positionally.
            "--project" => project = args.next(),
            // Options `dotnet build` would reject.
            "--launch-profile" | "-lp" => {
                args.next();
            }
            "--no-launch-profile" | "--no-build" => {}
            _ => match arg
                .strip_prefix("--project=")
                .or(arg.strip_prefix("--project:"))
            {
                Some(path) => project = Some(path.into()),
                None => build_args.push(arg),
            },
        }
    }
    let program_args: Vec<String> = args.collect();

    let template = TaskTemplate {
        args: ["build".to_owned()]
            .into_iter()
            .chain(project)
            .chain(build_args)
            .collect(),
        ..task
    };

    Some(DebugScenario {
        label,
        adapter,
        build: Some(BuildTaskDefinition::Template(
            BuildTaskDefinitionTemplatePayload {
                locator_name: Some(LOCATOR_NAME.into()),
                template,
            },
        )),
        // Zed merges this over the request returned by `run_locator`.
        config: json!({ "args": program_args }).to_string(),
        tcp_connection: None,
    })
}

/// Finds the assembly the `dotnet build` task produced.
pub fn run_locator(task: TaskTemplate) -> Result<DebugRequest> {
    let cwd = task
        .cwd
        .ok_or("the dotnet build task has no working directory")?;

    // Processes started by extensions don't run in the task's directory, so
    // the project has to be passed as an absolute path.
    let mut args = task.args;
    match args.get(1) {
        Some(project) if !project.starts_with('-') => args[1] = resolve(&cwd, project),
        _ => args.insert(1, cwd.clone()),
    }
    args.extend(["-getProperty:TargetPath".into(), "-nologo".into()]);

    let output = zed::process::Command::new("dotnet")
        .args(args.iter().cloned())
        .envs(task.env.iter().cloned())
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();

    if output.status != Some(0) {
        // MSBuild reports most errors on stdout.
        let stderr = String::from_utf8_lossy(&output.stderr);
        let reason = if stderr.trim().is_empty() {
            &stdout
        } else {
            stderr.trim()
        };
        return Err(format!("`dotnet {}` failed: {reason}", args.join(" ")));
    }
    if stdout.is_empty() {
        return Err(format!(
            "MSBuild reported no TargetPath for '{}'; for a multi-targeted project, pass `-f <framework>`",
            args[1]
        ));
    }

    Ok(DebugRequest::Launch(LaunchRequest {
        program: stdout,
        cwd: Some(cwd),
        args: Vec::new(),
        envs: task.env,
    }))
}

fn resolve(cwd: &str, path: &str) -> String {
    let is_absolute = path.starts_with(['/', '\\']) || path.get(1..2) == Some(":");
    if is_absolute {
        path.to_owned()
    } else {
        format!("{cwd}/{path}")
    }
}
