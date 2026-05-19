use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use runx_contracts::HarnessReceipt;
use runx_runtime::{LocalReceiptStore, ReceiptStoreError};
use serde_json::json;

const SUCCESS_FIXTURE: &str =
    include_str!("../../../fixtures/contracts/harness-spine/harness-receipt-success.json");
const ABNORMAL_FIXTURE: &str =
    include_str!("../../../fixtures/contracts/harness-spine/harness-receipt-abnormal.json");

#[test]
fn missing_store_fails_closed() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TestDir::new()?;
    let store = LocalReceiptStore::new(temp.path().join("missing"));

    let result = store.list();

    assert!(matches!(
        result,
        Err(ReceiptStoreError::MissingStore { .. })
    ));
    Ok(())
}

#[test]
fn file_instead_of_directory_is_typed_error() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TestDir::new()?;
    let store_path = temp.path().join("receipts");
    fs::write(&store_path, "not a directory")?;
    let store = LocalReceiptStore::new(&store_path);

    let result = store.list();

    assert!(matches!(
        result,
        Err(ReceiptStoreError::StoreNotDirectory { .. })
    ));
    Ok(())
}

#[test]
fn malformed_receipt_json_is_typed_error() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TestDir::new()?;
    fs::write(temp.path().join("hrn_rcpt_bad.json"), "{")?;
    let store = LocalReceiptStore::new(temp.path());

    let result = store.read_exact("hrn_rcpt_bad");

    assert!(matches!(
        result,
        Err(ReceiptStoreError::MalformedJson { .. })
    ));
    Ok(())
}

#[test]
fn wrong_receipt_schema_is_typed_error() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TestDir::new()?;
    write_json(
        temp.path(),
        "hrn_rcpt_wrong.json",
        &json!({
            "schema": "runx.not_harness_receipt.v1",
            "id": "hrn_rcpt_wrong"
        }),
    )?;
    let store = LocalReceiptStore::new(temp.path());

    let result = store.read_exact("hrn_rcpt_wrong");

    assert!(matches!(result, Err(ReceiptStoreError::WrongSchema { .. })));
    Ok(())
}

#[test]
fn receipt_id_must_match_file_name() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TestDir::new()?;
    write_json(temp.path(), "hrn_rcpt_other.json", &success_receipt()?)?;
    let store = LocalReceiptStore::new(temp.path());

    let result = store.read_exact("hrn_rcpt_other");

    assert!(matches!(
        result,
        Err(ReceiptStoreError::IdFilenameMismatch { .. })
    ));
    Ok(())
}

#[test]
fn exact_read_does_not_use_partial_or_suffix_lookup() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TestDir::new()?;
    write_json(temp.path(), "hrn_rcpt_123.json", &success_receipt()?)?;
    let store = LocalReceiptStore::new(temp.path());

    let result = store.read_exact("123");

    assert!(matches!(
        result,
        Err(ReceiptStoreError::MissingReceipt { .. })
    ));
    Ok(())
}

#[test]
fn exact_read_list_and_rebuild_index_succeed() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TestDir::new()?;
    write_json(temp.path(), "hrn_rcpt_123.json", &success_receipt()?)?;
    write_json(temp.path(), "hrn_rcpt_failed_1.json", &abnormal_receipt()?)?;
    fs::write(temp.path().join("notes.txt"), "ignored")?;
    let store = LocalReceiptStore::new(temp.path());

    let receipt = store.read_exact("hrn_rcpt_123")?;
    let listed = store.list()?;
    let index = store.rebuild_index()?;
    let loaded_index = store.load_index()?;

    assert_eq!(receipt.id, "hrn_rcpt_123");
    assert_eq!(
        listed
            .iter()
            .map(|receipt| receipt.id.as_str())
            .collect::<Vec<_>>(),
        vec!["hrn_rcpt_123", "hrn_rcpt_failed_1"]
    );
    assert_eq!(
        index
            .entries
            .iter()
            .map(|entry| entry.receipt_id.as_str())
            .collect::<Vec<_>>(),
        vec!["hrn_rcpt_123", "hrn_rcpt_failed_1"]
    );
    assert_eq!(index.entries[0].file_name, "hrn_rcpt_123.json");
    assert_eq!(loaded_index.entries, index.entries);
    assert!(temp.path().join("index.json").exists());
    Ok(())
}

#[test]
fn write_receipt_commits_readable_receipt_and_index() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TestDir::new()?;
    let store = LocalReceiptStore::new(temp.path().join("receipts"));
    let receipt = success_harness_receipt()?;

    store.write_receipt(&receipt)?;

    let stored = store.read_exact(&receipt.id)?;
    let index = store.load_index()?;
    assert_eq!(stored.id, receipt.id);
    assert_eq!(index.entries.len(), 1);
    assert_eq!(index.entries[0].receipt_id, receipt.id);
    assert!(store.root().join(format!("{}.json", receipt.id)).exists());
    assert!(store.root().join("index.json").exists());
    Ok(())
}

#[test]
fn write_receipt_allows_identical_and_rejects_divergent_rewrite()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = TestDir::new()?;
    let store = LocalReceiptStore::new(temp.path());
    let receipt = success_harness_receipt()?;
    let mut changed = receipt.clone();
    changed.signature.value = "sig:different".to_owned();

    store.write_receipt(&receipt)?;
    store.write_receipt(&receipt)?;
    let result = store.write_receipt(&changed);

    assert!(matches!(
        result,
        Err(ReceiptStoreError::ReceiptAlreadyExists { .. })
    ));
    Ok(())
}

#[test]
fn index_write_failure_reports_stale_but_receipt_stays_readable()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = TestDir::new()?;
    fs::create_dir(temp.path().join("index.json"))?;
    let store = LocalReceiptStore::new(temp.path());
    let receipt = success_harness_receipt()?;

    let result = store.write_receipt(&receipt);

    assert!(matches!(
        result,
        Err(ReceiptStoreError::ReceiptIndexStale { .. })
    ));
    assert_eq!(store.read_exact(&receipt.id)?.id, receipt.id);
    Ok(())
}

#[test]
fn malformed_index_is_typed_error() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TestDir::new()?;
    fs::write(temp.path().join("index.json"), "{")?;
    let store = LocalReceiptStore::new(temp.path());

    let result = store.load_index();

    assert!(matches!(
        result,
        Err(ReceiptStoreError::MalformedIndex { .. })
    ));
    Ok(())
}

#[test]
fn stale_index_is_typed_error() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TestDir::new()?;
    write_json(temp.path(), "hrn_rcpt_123.json", &success_receipt()?)?;
    write_json(
        temp.path(),
        "index.json",
        &json!({
            "schema": "runx.receipt_store_index.v1",
            "generated_at": "1",
            "entries": []
        }),
    )?;
    let store = LocalReceiptStore::new(temp.path());

    let result = store.load_index();

    assert!(matches!(
        result,
        Err(ReceiptStoreError::ReceiptIndexStale { .. })
    ));
    Ok(())
}

#[test]
fn receipt_store_error_display_does_not_leak_absolute_paths()
-> Result<(), Box<dyn std::error::Error>> {
    let external = PathBuf::from("/Users/kam/private/runx-receipts");
    let errors = [
        ReceiptStoreError::MissingStore {
            path: external.clone(),
        }
        .to_string(),
        ReceiptStoreError::MalformedIndex {
            path: external,
            message: "bad".to_owned(),
        }
        .to_string(),
    ];

    for message in errors {
        assert_redacts_external_path(&message);
    }
    Ok(())
}

#[test]
fn store_public_projection_redacts_external_absolute_root() {
    let workspace = PathBuf::from("/workspace/runx");
    let project_runx_dir = workspace.join(".runx");
    let external = PathBuf::from("/Users/kam/private/runx-receipts");
    let store = LocalReceiptStore::new(&external);

    let projection = store.public_projection(&workspace, &project_runx_dir);
    let summary = projection.summary();
    let label = projection.label().as_str();

    assert!(label.starts_with("external-receipt-store:"));
    assert!(summary.contains(label));
    assert_redacts_external_path(&summary);
}

#[test]
fn store_public_projection_uses_project_relative_root() {
    let workspace = PathBuf::from("/workspace/runx");
    let project_runx_dir = workspace.join(".runx");
    let store = LocalReceiptStore::new(project_runx_dir.join("receipts"));

    let projection = store.public_projection(&workspace, &project_runx_dir);

    assert_eq!(projection.label().as_str(), ".runx/receipts");
    assert_eq!(projection.summary(), "receipt store: .runx/receipts");
}

#[test]
fn receipt_store_error_public_message_uses_safe_label() {
    let workspace = PathBuf::from("/workspace/runx");
    let project_runx_dir = workspace.join(".runx");
    let external = PathBuf::from("/Users/kam/private/runx-receipts");
    let store = LocalReceiptStore::new(&external);
    let projection = store.public_projection(&workspace, &project_runx_dir);
    let error = ReceiptStoreError::MissingStore { path: external };

    let message = error.public_message(projection.label());

    assert!(message.contains(projection.label().as_str()));
    assert_redacts_external_path(&message);
}

#[test]
fn receipt_store_error_public_message_redacts_path_like_fields() {
    let workspace = PathBuf::from("/workspace/runx");
    let project_runx_dir = workspace.join(".runx");
    let external = PathBuf::from("/Users/kam/private/runx-receipts");
    let store = LocalReceiptStore::new(&external);
    let projection = store.public_projection(&workspace, &project_runx_dir);
    let invalid_id = ReceiptStoreError::InvalidReceiptId {
        receipt_id: external.to_string_lossy().into_owned(),
    };
    let mismatch = ReceiptStoreError::IdFilenameMismatch {
        path: external,
        receipt_id: "/Users/kam/private/runx-receipts".to_owned(),
        file_stem: "runx-receipts".to_owned(),
    };

    let invalid_id_message = invalid_id.public_message(projection.label());
    let mismatch_message = mismatch.public_message(projection.label());

    assert_redacts_external_path(&invalid_id_message);
    assert_redacts_external_path(&mismatch_message);
}

fn success_receipt() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    fixture_expected(SUCCESS_FIXTURE)
}

fn abnormal_receipt() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    fixture_expected(ABNORMAL_FIXTURE)
}

fn success_harness_receipt() -> Result<HarnessReceipt, Box<dyn std::error::Error>> {
    serde_json::from_value(success_receipt()?).map_err(Into::into)
}

fn fixture_expected(source: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let fixture: serde_json::Value = serde_json::from_str(source)?;
    fixture.get("expected").cloned().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "fixture is missing expected receipt",
        )
        .into()
    })
}

fn write_json(
    dir: &Path,
    file_name: &str,
    value: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(dir.join(file_name), serde_json::to_string(value)?)?;
    Ok(())
}

struct TestDir {
    path: PathBuf,
}

static NEXT_TEST_DIR: AtomicUsize = AtomicUsize::new(0);

impl TestDir {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let serial = NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runx-runtime-receipt-store-{}-{serial}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ignored = fs::remove_dir_all(&self.path);
    }
}

fn assert_redacts_external_path(text: &str) {
    assert!(!text.contains("/Users"));
    assert!(!text.contains("kam"));
    assert!(!text.contains("private"));
    assert!(!text.contains("runx-receipts"));
}
