// rust-style-allow: large-file - launcher argument parity is centralized for CLI routing tests.
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

pub const DEFAULT_NPM_PACKAGE: &str = "@runxhq/cli@latest";

#[derive(Debug, Eq, PartialEq)]
pub enum LauncherAction {
    Delegate(CommandPlan),
    Error(String),
    RunInit(InitPlan),
    RunNew(NewPlan),
    RunHistory(HistoryPlan),
    RunHarness(HarnessPlan),
    RunTool(ToolPlan),
    PrintHelp,
    PrintVersion,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CommandPlan {
    pub program: OsString,
    pub args: Vec<OsString>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct HarnessPlan {
    pub fixture_path: OsString,
}

#[derive(Debug, Eq, PartialEq)]
pub struct HistoryPlan {
    pub args: Vec<OsString>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct NewPlan {
    pub name: String,
    pub directory: Option<PathBuf>,
    pub json: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct InitPlan {
    pub global: bool,
    pub prefetch_official: bool,
    pub json: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ToolPlan {
    pub action: ToolAction,
    pub path: Option<PathBuf>,
    pub ref_or_query: Option<String>,
    pub all: bool,
    pub source: Option<String>,
    pub json: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ToolAction {
    Build,
    Search,
    Inspect,
}

pub fn plan_launcher(
    args: Vec<OsString>,
    npm_package: Option<OsString>,
    js_bin: Option<OsString>,
) -> LauncherAction {
    plan_launcher_with_rust_harness(args, npm_package, js_bin, None)
}

pub fn plan_launcher_with_rust_harness(
    args: Vec<OsString>,
    npm_package: Option<OsString>,
    js_bin: Option<OsString>,
    rust_harness: Option<OsString>,
) -> LauncherAction {
    if has_arg(&args, "--shim-version") {
        return LauncherAction::PrintVersion;
    }

    if has_arg(&args, "--shim-help") {
        return LauncherAction::PrintHelp;
    }

    if rust_harness_requested(rust_harness.as_deref()) && first_arg_is(&args, "harness") {
        return native_harness_plan(&args);
    }

    if first_arg_is(&args, "new") {
        return parse_new_plan(&args).map_or_else(LauncherAction::Error, LauncherAction::RunNew);
    }

    if first_arg_is(&args, "init") {
        return parse_init_plan(&args).map_or_else(LauncherAction::Error, LauncherAction::RunInit);
    }

    if first_arg_is(&args, "history") {
        return LauncherAction::RunHistory(HistoryPlan { args });
    }

    if first_arg_is(&args, "tool") && tool_subcommand_is_native(&args) {
        return parse_tool_plan(&args).map_or_else(LauncherAction::Error, LauncherAction::RunTool);
    }

    if let Some(js_bin) = non_empty_os(js_bin) {
        let mut planned_args = Vec::with_capacity(args.len() + 1);
        planned_args.push(js_bin);
        planned_args.extend(args);
        return LauncherAction::Delegate(CommandPlan {
            program: node_command().into(),
            args: planned_args,
        });
    }

    let package = non_empty_os(npm_package).unwrap_or_else(|| DEFAULT_NPM_PACKAGE.into());
    let mut planned_args = vec![
        "exec".into(),
        "--yes".into(),
        "--package".into(),
        package,
        "--".into(),
        "runx".into(),
    ];
    planned_args.extend(args);

    LauncherAction::Delegate(CommandPlan {
        program: npm_command().into(),
        args: planned_args,
    })
}

pub fn shim_help() -> String {
    format!(
        "\
runx Cargo launcher

Usage:
  runx [runx CLI args]
  runx --shim-version
  runx --shim-help

Environment:
  RUNX_NPM_PACKAGE  npm package spec to execute, defaults to {DEFAULT_NPM_PACKAGE}
  RUNX_JS_BIN       local JavaScript runx entrypoint to execute with node
  RUNX_RUST_HARNESS opt into native Rust `runx harness <fixture>` replay

Native commands:
  runx new <name> [--directory dir] [--json]
  runx init [-g|--global] [--prefetch official] [--json]
  runx history [query] [--skill s] [--status s] [--source s] [--actor a] [--artifact-type t] [--since iso] [--until iso] [--receipt-dir dir] [--json]
  runx tool build <tool-dir>|--all [--json]
  runx tool search <query> [--source source] [--json]
  runx tool inspect <ref> [--source source] [--json]
"
    )
}

pub fn npm_command() -> &'static str {
    if cfg!(windows) { "npm.cmd" } else { "npm" }
}

pub fn node_command() -> &'static str {
    if cfg!(windows) { "node.exe" } else { "node" }
}

fn has_arg(args: &[OsString], expected: &str) -> bool {
    args.iter().any(|arg| arg == OsStr::new(expected))
}

fn first_arg_is(args: &[OsString], expected: &str) -> bool {
    args.first().is_some_and(|arg| arg == OsStr::new(expected))
}

fn non_empty_os(value: Option<OsString>) -> Option<OsString> {
    value.filter(|value| !value.is_empty())
}

fn rust_harness_requested(value: Option<&OsStr>) -> bool {
    value.is_some_and(|value| !value.is_empty() && value != OsStr::new("0"))
}

fn native_harness_plan(args: &[OsString]) -> LauncherAction {
    if args.len() != 2 {
        return LauncherAction::Error(
            "runx harness requires exactly one fixture path when RUNX_RUST_HARNESS is set"
                .to_owned(),
        );
    }
    LauncherAction::RunHarness(HarnessPlan {
        fixture_path: args[1].clone(),
    })
}

fn parse_new_plan(args: &[OsString]) -> Result<NewPlan, String> {
    let mut name = None;
    let mut directory = None;
    let mut json = false;
    let mut positional_directory = None;
    let mut extra_positionals = Vec::new();
    let mut index = 1;

    while index < args.len() {
        let token = os_arg(args, index, "new")?;
        if !token.starts_with("--") {
            if name.is_none() {
                name = Some(token.to_owned());
            } else if positional_directory.is_none() {
                positional_directory = Some(PathBuf::from(token));
            } else {
                extra_positionals.push(token.to_owned());
            }
            index += 1;
            continue;
        }

        let (flag, inline_value) = split_flag(token);
        match flag {
            "--json" => {
                if inline_value.is_some() {
                    return Err("--json does not take a value".to_owned());
                }
                json = true;
                index += 1;
            }
            "--directory" | "--dir" => {
                let (value, next_index) = flag_value(args, index, flag, inline_value, "new")?;
                directory = Some(PathBuf::from(value));
                index = next_index;
            }
            _ => return Err(format!("unknown new flag {flag}")),
        }
    }

    if !extra_positionals.is_empty() {
        return Err("runx new accepts at most one directory argument".to_owned());
    }

    Ok(NewPlan {
        name: name.ok_or_else(|| "runx new requires a package name".to_owned())?,
        directory: directory.or(positional_directory),
        json,
    })
}

fn parse_init_plan(args: &[OsString]) -> Result<InitPlan, String> {
    let mut global = false;
    let mut prefetch_official = false;
    let mut json = false;
    let mut index = 1;

    while index < args.len() {
        let token = os_arg(args, index, "init")?;
        if token == "-g" {
            global = true;
            index += 1;
            continue;
        }
        if !token.starts_with("--") {
            return Err(format!("unexpected init argument {token}"));
        }

        let (flag, inline_value) = split_flag(token);
        match flag {
            "--json" => {
                if inline_value.is_some() {
                    return Err("--json does not take a value".to_owned());
                }
                json = true;
                index += 1;
            }
            "--global" => {
                if inline_value.is_some() {
                    return Err("--global does not take a value".to_owned());
                }
                global = true;
                index += 1;
            }
            "--prefetch" | "--prefetchOfficial" | "--prefetch-official" => {
                if matches!(inline_value, Some("false") | Some("0")) {
                    prefetch_official = false;
                    index += 1;
                    continue;
                }
                let (value, next_index) = optional_flag_value(args, index, inline_value, "init")?;
                prefetch_official = value.is_none_or(|value| truthy(&value));
                index = next_index;
            }
            _ => return Err(format!("unknown init flag {flag}")),
        }
    }

    Ok(InitPlan {
        global,
        prefetch_official,
        json,
    })
}

fn tool_subcommand_is_native(args: &[OsString]) -> bool {
    matches!(
        args.get(1).and_then(|arg| arg.to_str()),
        Some("build" | "search" | "inspect")
    )
}

fn parse_tool_plan(args: &[OsString]) -> Result<ToolPlan, String> {
    let subcommand = os_arg(args, 1, "tool")?;
    let action = match subcommand {
        "build" => ToolAction::Build,
        "search" => ToolAction::Search,
        "inspect" => ToolAction::Inspect,
        _ => return Err(format!("unknown tool subcommand {subcommand}")),
    };
    let mut json = false;
    let mut all = false;
    let mut source = None;
    let mut positionals = Vec::new();
    let mut index = 2;

    while index < args.len() {
        let token = os_arg(args, index, "tool")?;
        if !token.starts_with("--") {
            positionals.push(token.to_owned());
            index += 1;
            continue;
        }

        let (flag, inline_value) = split_flag(token);
        match flag {
            "--json" => {
                if inline_value.is_some() {
                    return Err("--json does not take a value".to_owned());
                }
                json = true;
                index += 1;
            }
            "--all" => {
                if inline_value.is_some() {
                    return Err("--all does not take a value".to_owned());
                }
                all = true;
                index += 1;
            }
            "--source" => {
                let (value, next_index) = flag_value(args, index, flag, inline_value, "tool")?;
                source = Some(value);
                index = next_index;
            }
            _ => return Err(format!("unknown tool flag {flag}")),
        }
    }

    match action {
        ToolAction::Build => build_tool_plan(positionals, all, source, json),
        ToolAction::Search => search_tool_plan(positionals, all, source, json),
        ToolAction::Inspect => inspect_tool_plan(positionals, all, source, json),
    }
}

fn build_tool_plan(
    positionals: Vec<String>,
    all: bool,
    source: Option<String>,
    json: bool,
) -> Result<ToolPlan, String> {
    if source.is_some() {
        return Err("runx tool build does not accept --source".to_owned());
    }
    if all && !positionals.is_empty() {
        return Err("runx tool build accepts either --all or one tool directory".to_owned());
    }
    if !all && positionals.len() != 1 {
        return Err("runx tool build requires a tool directory or --all".to_owned());
    }

    Ok(ToolPlan {
        action: ToolAction::Build,
        path: positionals.first().map(PathBuf::from),
        ref_or_query: None,
        all,
        source: None,
        json,
    })
}

fn search_tool_plan(
    positionals: Vec<String>,
    all: bool,
    source: Option<String>,
    json: bool,
) -> Result<ToolPlan, String> {
    if all {
        return Err("runx tool search does not accept --all".to_owned());
    }
    let query = positionals.join(" ");
    if query.is_empty() {
        return Err("runx tool search requires a query".to_owned());
    }

    Ok(ToolPlan {
        action: ToolAction::Search,
        path: None,
        ref_or_query: Some(query),
        all: false,
        source,
        json,
    })
}

fn inspect_tool_plan(
    positionals: Vec<String>,
    all: bool,
    source: Option<String>,
    json: bool,
) -> Result<ToolPlan, String> {
    if all {
        return Err("runx tool inspect does not accept --all".to_owned());
    }
    let tool_ref = positionals.join(" ");
    if tool_ref.is_empty() {
        return Err("runx tool inspect requires a tool reference".to_owned());
    }

    Ok(ToolPlan {
        action: ToolAction::Inspect,
        path: None,
        ref_or_query: Some(tool_ref),
        all: false,
        source,
        json,
    })
}

fn os_arg<'a>(args: &'a [OsString], index: usize, command: &str) -> Result<&'a str, String> {
    args.get(index)
        .and_then(|arg| arg.to_str())
        .ok_or_else(|| format!("{command} arguments must be UTF-8"))
}

fn split_flag(token: &str) -> (&str, Option<&str>) {
    token
        .split_once('=')
        .map_or((token, None), |(flag, value)| (flag, Some(value)))
}

fn flag_value(
    args: &[OsString],
    index: usize,
    flag: &str,
    inline_value: Option<&str>,
    command: &str,
) -> Result<(String, usize), String> {
    if let Some(value) = inline_value {
        return Ok((value.to_owned(), index + 1));
    }
    let value = os_arg(args, index + 1, command).map_err(|_| format!("{flag} requires a value"))?;
    if value.starts_with("--") {
        return Err(format!("{flag} requires a value"));
    }
    Ok((value.to_owned(), index + 2))
}

fn optional_flag_value(
    args: &[OsString],
    index: usize,
    inline_value: Option<&str>,
    command: &str,
) -> Result<(Option<String>, usize), String> {
    if let Some(value) = inline_value {
        return Ok((Some(value.to_owned()), index + 1));
    }
    let Some(value) = args.get(index + 1).and_then(|arg| arg.to_str()) else {
        return Ok((None, index + 1));
    };
    if value.starts_with('-') {
        return Ok((None, index + 1));
    }
    os_arg(args, index + 1, command)?;
    Ok((Some(value.to_owned()), index + 2))
}

fn truthy(value: &str) -> bool {
    matches!(value, "true" | "1" | "yes" | "official")
}
