use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;
use std::{collections::BTreeMap, env};

#[derive(Debug, Eq, PartialEq)]
pub struct McpPlan {
    pub refs: Vec<PathBuf>,
    pub receipt_dir: Option<PathBuf>,
    pub runner: Option<String>,
    /// When set, serve the governed MCP server over streamable HTTP at this
    /// address instead of over stdio.
    pub http_listen: Option<String>,
}

pub fn parse_mcp_plan(args: &[OsString]) -> Result<McpPlan, String> {
    let subcommand = os_arg(args, 1)?;
    if subcommand != "serve" {
        return Err(format!("unknown mcp subcommand {subcommand}"));
    }
    let mut refs = Vec::new();
    let mut receipt_dir = None;
    let mut runner = None;
    let mut http_listen = None;
    let mut index = 2;
    while index < args.len() {
        let token = os_arg(args, index)?;
        if !token.starts_with("--") {
            refs.push(PathBuf::from(token));
            index += 1;
            continue;
        }
        let (flag, inline_value) = split_flag(token);
        match flag {
            "--receipt-dir" => {
                let (value, next_index) = flag_value(args, index, flag, inline_value)?;
                receipt_dir = Some(PathBuf::from(value));
                index = next_index;
            }
            "--runner" => {
                let (value, next_index) = flag_value(args, index, flag, inline_value)?;
                runner = Some(value);
                index = next_index;
            }
            "--http-listen" => {
                let (value, next_index) = flag_value(args, index, flag, inline_value)?;
                http_listen = Some(value);
                index = next_index;
            }
            _ => return Err(format!("unknown mcp serve flag {flag}")),
        }
    }
    if refs.is_empty() {
        return Err("runx mcp serve requires at least one skill reference.".to_owned());
    }
    Ok(McpPlan {
        refs,
        receipt_dir,
        runner,
        http_listen,
    })
}

pub fn run_native_mcp(plan: McpPlan) -> ExitCode {
    let options =
        match runx_runtime::adapters::mcp::McpServerOptions::from_skill_paths_with_execution(
            &plan.refs,
            "runx-cli",
            env!("CARGO_PKG_VERSION"),
            runx_runtime::adapters::mcp::McpServerExecutionOptions {
                runner: plan.runner,
                receipt_dir: plan.receipt_dir,
                env: mcp_execution_env(),
            },
        ) {
            Ok(options) => options,
            Err(error) => {
                let _ignored = writeln!(std::io::stderr(), "runx: {error}");
                return ExitCode::from(1);
            }
        };
    let result = match &plan.http_listen {
        Some(listen_addr) => {
            runx_runtime::adapters::mcp::serve_mcp_http_server_blocking(listen_addr, options)
        }
        None => runx_runtime::adapters::mcp::serve_mcp_json_rpc(
            std::io::stdin(),
            std::io::stdout(),
            options,
        ),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ignored = writeln!(std::io::stderr(), "runx: {error}");
            ExitCode::from(1)
        }
    }
}

fn mcp_execution_env() -> BTreeMap<String, String> {
    let mut env = env::vars().collect::<BTreeMap<_, _>>();
    if !env.contains_key(runx_runtime::RUNX_CWD_ENV)
        && let Ok(cwd) = env::current_dir()
    {
        env.insert(
            runx_runtime::RUNX_CWD_ENV.to_owned(),
            cwd.to_string_lossy().into_owned(),
        );
    }
    env
}

fn os_arg(args: &[OsString], index: usize) -> Result<&str, String> {
    args.get(index)
        .and_then(|arg| arg.to_str())
        .ok_or_else(|| "mcp arguments must be UTF-8".to_owned())
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
) -> Result<(String, usize), String> {
    if let Some(value) = inline_value {
        return Ok((value.to_owned(), index + 1));
    }
    let value = os_arg(args, index + 1).map_err(|_| format!("{flag} requires a value"))?;
    if value.starts_with("--") {
        return Err(format!("{flag} requires a value"));
    }
    Ok((value.to_owned(), index + 2))
}
