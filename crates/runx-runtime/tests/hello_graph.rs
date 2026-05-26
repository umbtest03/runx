#![cfg(feature = "cli-tool")]

use std::path::Path;

use runx_core::state_machine::GraphStatus;
use runx_receipts::validate_receipt_tree;
use runx_runtime::adapters::cli_tool::CliToolAdapter;
use runx_runtime::{
    RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64_ENV, RUNX_RECEIPT_SIGN_ISSUER_TYPE_ENV,
    RUNX_RECEIPT_SIGN_KID_ENV, Runtime, RuntimeOptions,
};

#[test]
fn hello_graph_runs_to_receipt_tree() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = Runtime::new(CliToolAdapter, signed_runtime_options()?);
    let run = runtime.run_graph_file(Path::new("../../examples/hello-graph/graph.yaml"))?;

    assert_eq!(run.graph.name, "hello-graph");
    assert_eq!(run.state.status, GraphStatus::Succeeded);
    assert_eq!(
        run.steps
            .iter()
            .map(|step| step.step_id.as_str())
            .collect::<Vec<_>>(),
        vec!["first", "second"]
    );
    assert_eq!(run.steps[0].output.stdout, "hello from graph\n");
    assert!(run.steps[1].output.stdout.starts_with("hello from graph"));

    let children = run
        .steps
        .iter()
        .map(|step| step.receipt.clone())
        .collect::<Vec<_>>();
    assert!(validate_receipt_tree(&run.receipt, &children).is_ok());
    Ok(())
}

#[test]
fn hello_graph_resumes_from_checkpoint() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = Runtime::new(CliToolAdapter, RuntimeOptions::local_development());
    let graph_path = Path::new("../../examples/hello-graph/graph.yaml");

    let checkpoint = runtime.run_graph_file_until_steps(graph_path, 1)?;
    assert_eq!(checkpoint.steps.len(), 1);
    assert_eq!(checkpoint.steps[0].step_id, "first");

    let run = runtime.resume_graph_file(graph_path, checkpoint)?;
    assert_eq!(run.state.status, GraphStatus::Succeeded);
    assert_eq!(
        run.steps
            .iter()
            .map(|step| step.step_id.as_str())
            .collect::<Vec<_>>(),
        vec!["first", "second"]
    );
    Ok(())
}

fn signed_runtime_options() -> Result<RuntimeOptions, runx_runtime::RuntimeError> {
    let mut env = std::env::vars().collect::<std::collections::BTreeMap<_, _>>();
    env.insert(
        RUNX_RECEIPT_SIGN_KID_ENV.to_owned(),
        "runx-runtime-prod-fixture-key".to_owned(),
    );
    env.insert(
        RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64_ENV.to_owned(),
        "QkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkI=".to_owned(),
    );
    env.insert(
        RUNX_RECEIPT_SIGN_ISSUER_TYPE_ENV.to_owned(),
        "hosted".to_owned(),
    );
    RuntimeOptions::from_env(env)
}
