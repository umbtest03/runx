use std::path::PathBuf;

use runx_cli::launcher::{
    CommandPlan, DEFAULT_NPM_PACKAGE, HarnessPlan, HistoryPlan, InitPlan, LauncherAction, NewPlan,
    ToolAction, ToolPlan, node_command, npm_command, plan_launcher,
    plan_launcher_with_rust_harness,
};

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

#[test]
fn rust_harness_signal_routes_harness_command_to_native_runner() {
    let action = plan_launcher_with_rust_harness(
        vec!["harness".into(), "fixtures/harness/echo-skill.yaml".into()],
        Some("@runxhq/cli@0.5.22".into()),
        Some("/repo/oss/packages/cli/bin/runx.js".into()),
        Some("1".into()),
    );

    assert_eq!(
        action,
        LauncherAction::RunHarness(HarnessPlan {
            fixture_path: "fixtures/harness/echo-skill.yaml".into(),
        })
    );
}

#[test]
fn harness_without_rust_signal_still_delegates() {
    let action = plan_launcher(vec!["harness".into(), "fixture.yaml".into()], None, None);

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
                "harness".into(),
                "fixture.yaml".into(),
            ],
        })
    );
}

#[test]
fn history_routes_to_native_cli_even_with_js_fallback_configured() {
    let action = plan_launcher(
        vec![
            "history".into(),
            "sourcey".into(),
            "--json".into(),
            "--receipt-dir".into(),
            ".runx/receipts".into(),
        ],
        Some("@runxhq/cli@0.5.22".into()),
        Some("/repo/oss/packages/cli/bin/runx.js".into()),
    );

    assert_eq!(
        action,
        LauncherAction::RunHistory(HistoryPlan {
            args: vec![
                "history".into(),
                "sourcey".into(),
                "--json".into(),
                "--receipt-dir".into(),
                ".runx/receipts".into(),
            ],
        })
    );
}

#[test]
fn new_routes_to_native_cli_even_with_js_fallback_configured() {
    let action = plan_launcher(
        vec![
            "new".into(),
            "docs-demo".into(),
            "--directory".into(),
            "tmp/docs-demo".into(),
            "--json".into(),
        ],
        Some("@runxhq/cli@0.5.22".into()),
        Some("/repo/oss/packages/cli/bin/runx.js".into()),
    );

    assert_eq!(
        action,
        LauncherAction::RunNew(NewPlan {
            name: "docs-demo".to_owned(),
            directory: Some(PathBuf::from("tmp/docs-demo")),
            json: true,
        })
    );
}

#[test]
fn new_accepts_positional_directory() {
    let action = plan_launcher(
        vec!["new".into(), "docs-demo".into(), "out".into()],
        None,
        None,
    );

    assert_eq!(
        action,
        LauncherAction::RunNew(NewPlan {
            name: "docs-demo".to_owned(),
            directory: Some(PathBuf::from("out")),
            json: false,
        })
    );
}

#[test]
fn init_routes_to_native_cli_even_with_js_fallback_configured() {
    let action = plan_launcher(
        vec![
            "init".into(),
            "-g".into(),
            "--prefetch".into(),
            "official".into(),
            "--json".into(),
        ],
        Some("@runxhq/cli@0.5.22".into()),
        Some("/repo/oss/packages/cli/bin/runx.js".into()),
    );

    assert_eq!(
        action,
        LauncherAction::RunInit(InitPlan {
            global: true,
            prefetch_official: true,
            json: true,
        })
    );
}

#[test]
fn tool_build_routes_to_native_cli_even_with_js_fallback_configured() {
    let action = plan_launcher(
        vec![
            "tool".into(),
            "build".into(),
            "tools/docs/echo".into(),
            "--json".into(),
        ],
        Some("@runxhq/cli@0.5.22".into()),
        Some("/repo/oss/packages/cli/bin/runx.js".into()),
    );

    assert_eq!(
        action,
        LauncherAction::RunTool(ToolPlan {
            action: ToolAction::Build,
            path: Some(PathBuf::from("tools/docs/echo")),
            ref_or_query: None,
            all: false,
            source: None,
            json: true,
        })
    );
}

#[test]
fn tool_build_all_routes_to_native_cli() {
    let action = plan_launcher(
        vec!["tool".into(), "build".into(), "--all".into()],
        None,
        None,
    );

    assert_eq!(
        action,
        LauncherAction::RunTool(ToolPlan {
            action: ToolAction::Build,
            path: None,
            ref_or_query: None,
            all: true,
            source: None,
            json: false,
        })
    );
}

#[test]
fn tool_search_routes_to_native_cli_even_with_js_fallback_configured() {
    let action = plan_launcher(
        vec![
            "tool".into(),
            "search".into(),
            "echo".into(),
            "writer".into(),
            "--source".into(),
            "fixture-mcp".into(),
            "--json".into(),
        ],
        Some("@runxhq/cli@0.5.22".into()),
        Some("/repo/oss/packages/cli/bin/runx.js".into()),
    );

    assert_eq!(
        action,
        LauncherAction::RunTool(ToolPlan {
            action: ToolAction::Search,
            path: None,
            ref_or_query: Some("echo writer".to_owned()),
            all: false,
            source: Some("fixture-mcp".to_owned()),
            json: true,
        })
    );
}

#[test]
fn tool_inspect_routes_to_native_cli_even_with_js_fallback_configured() {
    let action = plan_launcher(
        vec![
            "tool".into(),
            "inspect".into(),
            "fixture.echo".into(),
            "--source=fixture-mcp".into(),
        ],
        Some("@runxhq/cli@0.5.22".into()),
        Some("/repo/oss/packages/cli/bin/runx.js".into()),
    );

    assert_eq!(
        action,
        LauncherAction::RunTool(ToolPlan {
            action: ToolAction::Inspect,
            path: None,
            ref_or_query: Some("fixture.echo".to_owned()),
            all: false,
            source: Some("fixture-mcp".to_owned()),
            json: false,
        })
    );
}

#[test]
fn tool_unknown_subcommand_still_delegates() {
    let action = plan_launcher(
        vec!["tool".into(), "publish".into(), "fixture.echo".into()],
        None,
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
                DEFAULT_NPM_PACKAGE.into(),
                "--".into(),
                "runx".into(),
                "tool".into(),
                "publish".into(),
                "fixture.echo".into(),
            ],
        })
    );
}

#[test]
fn init_accepts_truthy_prefetch_forms() {
    assert_eq!(
        plan_launcher(vec!["init".into(), "--prefetch".into()], None, None),
        LauncherAction::RunInit(InitPlan {
            global: false,
            prefetch_official: true,
            json: false,
        })
    );
    assert_eq!(
        plan_launcher(vec!["init".into(), "--prefetchOfficial".into()], None, None),
        LauncherAction::RunInit(InitPlan {
            global: false,
            prefetch_official: true,
            json: false,
        })
    );
}

#[test]
fn rust_harness_signal_rejects_unsupported_argument_shape() {
    let action =
        plan_launcher_with_rust_harness(vec!["harness".into()], None, None, Some("1".into()));

    assert_eq!(
        action,
        LauncherAction::Error(
            "runx harness requires exactly one fixture path when RUNX_RUST_HARNESS is set"
                .to_owned(),
        )
    );
}
