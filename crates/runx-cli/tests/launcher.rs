use runx_cli::config::{ConfigAction, ConfigPlan};
use runx_cli::connect::{ConnectAction, ConnectAuthorityKind, ConnectPlan};
use std::path::PathBuf;

use runx_cli::launcher::{
    CommandPlan, DEFAULT_NPM_PACKAGE, DoctorPlan, HarnessPlan, HistoryPlan, InitPlan,
    LauncherAction, ListKind, ListPlan, NativeLauncherOptions, NewPlan, ToolAction, ToolPlan,
    node_command, npm_command, plan_launcher, plan_launcher_with_native_options,
    plan_launcher_with_rust_harness,
};

fn plan_with_rust_cli(args: Vec<std::ffi::OsString>) -> LauncherAction {
    plan_launcher_with_native_options(
        args,
        None,
        None,
        NativeLauncherOptions {
            rust_cli: Some("1".into()),
            rust_harness: None,
        },
    )
}

fn plan_with_rust_cli_and_js(args: Vec<std::ffi::OsString>) -> LauncherAction {
    plan_launcher_with_native_options(
        args,
        Some("@runxhq/cli@0.5.22".into()),
        Some("/repo/oss/packages/cli/bin/runx.js".into()),
        NativeLauncherOptions {
            rust_cli: Some("1".into()),
            rust_harness: None,
        },
    )
}

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
fn connect_delegates_without_rust_cli_signal_even_with_js_fallback_configured() {
    let action = plan_launcher(
        vec!["connect".into(), "list".into(), "--json".into()],
        Some("@runxhq/cli@0.5.22".into()),
        Some("/repo/oss/packages/cli/bin/runx.js".into()),
    );

    assert_eq!(
        action,
        LauncherAction::Delegate(CommandPlan {
            program: node_command().into(),
            args: vec![
                "/repo/oss/packages/cli/bin/runx.js".into(),
                "connect".into(),
                "list".into(),
                "--json".into(),
            ],
        })
    );
}

#[test]
fn rust_cli_signal_routes_connect_to_native_plan() {
    let action = plan_with_rust_cli(vec!["connect".into(), "list".into(), "--json".into()]);

    assert_eq!(
        action,
        LauncherAction::RunConnect(ConnectPlan {
            action: ConnectAction::List,
            provider: None,
            grant_id: None,
            scopes: Vec::new(),
            scope_family: None,
            authority_kind: None,
            target_repo: None,
            target_locator: None,
            json: true,
        })
    );
}

#[test]
fn connect_routes_provider_command_to_native_plan() {
    let action = plan_with_rust_cli_and_js(vec![
        "connect".into(),
        "github".into(),
        "--scope".into(),
        "repo:read,checks:read".into(),
        "--scope-family".into(),
        "github_repo".into(),
        "--authority-kind".into(),
        "read_only".into(),
        "--target-repo".into(),
        "runxhq/aster".into(),
        "--json".into(),
    ]);

    assert_eq!(
        action,
        LauncherAction::RunConnect(ConnectPlan {
            action: ConnectAction::Preprovision,
            provider: Some("github".to_owned()),
            grant_id: None,
            scopes: vec!["repo:read".to_owned(), "checks:read".to_owned()],
            scope_family: Some("github_repo".to_owned()),
            authority_kind: Some(ConnectAuthorityKind::ReadOnly),
            target_repo: Some("runxhq/aster".to_owned()),
            target_locator: None,
            json: true,
        })
    );
}

#[test]
fn connect_rejects_invalid_connect_shape() {
    let action = plan_with_rust_cli(vec!["connect".into(), "revoke".into()]);

    assert_eq!(
        action,
        LauncherAction::Error("runx connect revoke requires exactly one grant id".to_owned())
    );
}

#[test]
fn config_routes_to_native_cli_even_with_js_fallback_configured() {
    let action = plan_with_rust_cli_and_js(vec![
        "config".into(),
        "set".into(),
        "agent.model".into(),
        "gpt-test".into(),
        "--json".into(),
    ]);

    assert_eq!(
        action,
        LauncherAction::RunConfig(ConfigPlan {
            action: ConfigAction::Set,
            key: Some("agent.model".to_owned()),
            value: Some("gpt-test".to_owned()),
            json: true,
        })
    );
}

#[test]
fn config_delegates_without_rust_cli_signal() {
    let action = plan_launcher(
        vec!["config".into(), "list".into(), "--json".into()],
        Some("@runxhq/cli@0.5.22".into()),
        Some("/repo/oss/packages/cli/bin/runx.js".into()),
    );

    assert_eq!(
        action,
        LauncherAction::Delegate(CommandPlan {
            program: node_command().into(),
            args: vec![
                "/repo/oss/packages/cli/bin/runx.js".into(),
                "config".into(),
                "list".into(),
                "--json".into(),
            ],
        })
    );
}

#[test]
fn doctor_routes_to_native_cli_even_with_js_fallback_configured() {
    let action = plan_with_rust_cli_and_js(vec![
        "doctor".into(),
        "fixtures/doctor/empty-success/workspace".into(),
        "--json".into(),
    ]);

    assert_eq!(
        action,
        LauncherAction::RunDoctor(DoctorPlan {
            path: Some(PathBuf::from("fixtures/doctor/empty-success/workspace")),
            json: true,
        })
    );
}

#[test]
fn doctor_repair_semantics_still_delegate_to_js() {
    let action = plan_with_rust_cli_and_js(vec!["doctor".into(), "--fix".into()]);

    assert_eq!(
        action,
        LauncherAction::Delegate(CommandPlan {
            program: node_command().into(),
            args: vec![
                "/repo/oss/packages/cli/bin/runx.js".into(),
                "doctor".into(),
                "--fix".into(),
            ],
        })
    );
}

#[test]
fn history_routes_to_native_cli_even_with_js_fallback_configured() {
    let action = plan_with_rust_cli_and_js(vec![
        "history".into(),
        "sourcey".into(),
        "--json".into(),
        "--receipt-dir".into(),
        ".runx/receipts".into(),
    ]);

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
fn list_routes_native_supported_shape_even_with_js_fallback_configured() {
    let action = plan_with_rust_cli_and_js(vec![
        "list".into(),
        "packets".into(),
        "--ok-only".into(),
        "--json".into(),
    ]);

    assert_eq!(
        action,
        LauncherAction::RunList(ListPlan {
            kind: ListKind::Packets,
            ok_only: true,
            invalid_only: false,
            json: true,
        })
    );
}

#[test]
fn list_delegates_unsupported_shape() {
    let action = plan_with_rust_cli_and_js(vec![
        "list".into(),
        "skills".into(),
        "--source".into(),
        "registry".into(),
    ]);

    assert_eq!(
        action,
        LauncherAction::Delegate(CommandPlan {
            program: node_command().into(),
            args: vec![
                "/repo/oss/packages/cli/bin/runx.js".into(),
                "list".into(),
                "skills".into(),
                "--source".into(),
                "registry".into(),
            ],
        })
    );
}

#[test]
fn list_rejects_conflicting_status_filters() {
    let action = plan_with_rust_cli(vec![
        "list".into(),
        "--ok-only".into(),
        "--invalid-only".into(),
    ]);

    assert_eq!(
        action,
        LauncherAction::Error("runx list accepts either --ok-only or --invalid-only".to_owned())
    );
}

#[test]
fn new_routes_to_native_cli_even_with_js_fallback_configured() {
    let action = plan_with_rust_cli_and_js(vec![
        "new".into(),
        "docs-demo".into(),
        "--directory".into(),
        "tmp/docs-demo".into(),
        "--json".into(),
    ]);

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
    let action = plan_with_rust_cli(vec!["new".into(), "docs-demo".into(), "out".into()]);

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
    let action = plan_with_rust_cli_and_js(vec![
        "init".into(),
        "-g".into(),
        "--prefetch".into(),
        "official".into(),
        "--json".into(),
    ]);

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
    let action = plan_with_rust_cli_and_js(vec![
        "tool".into(),
        "build".into(),
        "tools/docs/echo".into(),
        "--json".into(),
    ]);

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
    let action = plan_with_rust_cli(vec!["tool".into(), "build".into(), "--all".into()]);

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
    let action = plan_with_rust_cli_and_js(vec![
        "tool".into(),
        "search".into(),
        "echo".into(),
        "writer".into(),
        "--source".into(),
        "fixture-mcp".into(),
        "--json".into(),
    ]);

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
    let action = plan_with_rust_cli_and_js(vec![
        "tool".into(),
        "inspect".into(),
        "fixture.echo".into(),
        "--source=fixture-mcp".into(),
    ]);

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
        plan_with_rust_cli(vec!["init".into(), "--prefetch".into()]),
        LauncherAction::RunInit(InitPlan {
            global: false,
            prefetch_official: true,
            json: false,
        })
    );
    assert_eq!(
        plan_with_rust_cli(vec!["init".into(), "--prefetchOfficial".into()]),
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
