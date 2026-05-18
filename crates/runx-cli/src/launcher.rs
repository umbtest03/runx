use std::ffi::{OsStr, OsString};

pub const DEFAULT_NPM_PACKAGE: &str = "@runxhq/cli@latest";

#[derive(Debug, Eq, PartialEq)]
pub enum LauncherAction {
    Delegate(CommandPlan),
    PrintHelp,
    PrintVersion,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CommandPlan {
    pub program: OsString,
    pub args: Vec<OsString>,
}

pub fn plan_launcher(
    args: Vec<OsString>,
    npm_package: Option<OsString>,
    js_bin: Option<OsString>,
) -> LauncherAction {
    if has_arg(&args, "--shim-version") {
        return LauncherAction::PrintVersion;
    }

    if has_arg(&args, "--shim-help") {
        return LauncherAction::PrintHelp;
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

fn non_empty_os(value: Option<OsString>) -> Option<OsString> {
    value.filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_latest_npm_cli_package() {
        let action = plan_launcher(vec!["--help".into()], None, None);

        assert_eq!(
            action,
            LauncherAction::Delegate(CommandPlan {
                program: npm_command().into(),
                args: vec![
                    "exec".into(),
                    "--yes".into(),
                    "--package".into(),
                    DEFAULT_NPM_PACKAGE.into(),
                    "--".into(),
                    "runx".into(),
                    "--help".into(),
                ],
            })
        );
    }

    #[test]
    fn accepts_pinned_npm_package() {
        let action = plan_launcher(
            vec!["skill".into(), "sourcey".into()],
            Some("@runxhq/cli@0.5.22".into()),
            None,
        );

        assert_eq!(
            action,
            LauncherAction::Delegate(CommandPlan {
                program: npm_command().into(),
                args: vec![
                    "exec".into(),
                    "--yes".into(),
                    "--package".into(),
                    "@runxhq/cli@0.5.22".into(),
                    "--".into(),
                    "runx".into(),
                    "skill".into(),
                    "sourcey".into(),
                ],
            })
        );
    }

    #[test]
    fn local_js_bin_overrides_npm_package() {
        let action = plan_launcher(
            vec!["--help".into()],
            Some("@runxhq/cli@0.5.22".into()),
            Some("/repo/oss/packages/cli/bin/runx.js".into()),
        );

        assert_eq!(
            action,
            LauncherAction::Delegate(CommandPlan {
                program: node_command().into(),
                args: vec!["/repo/oss/packages/cli/bin/runx.js".into(), "--help".into()],
            })
        );
    }

    #[test]
    fn shim_flags_do_not_delegate() {
        assert_eq!(
            plan_launcher(vec!["--shim-version".into()], None, None),
            LauncherAction::PrintVersion
        );
        assert_eq!(
            plan_launcher(vec!["--shim-help".into()], None, None),
            LauncherAction::PrintHelp
        );
    }
}
