use std::collections::BTreeMap;

use runx_contracts::{HarnessReceipt, Reference, ReferenceType};
use runx_receipts::{
    ReceiptResolveResult, ReceiptResolver, ReceiptTreeConfig, ReceiptVerification, ResolvedReceipt,
    verify_receipt_tree_with_resolver,
};
use serde::Deserialize;

const ORACLE: &str = include_str!("../../../fixtures/runtime/receipt-tree/oracle.json");

#[derive(Debug, Deserialize)]
struct Oracle {
    cases: Vec<TreeCase>,
    receipts: BTreeMap<String, HarnessReceipt>,
}

#[derive(Debug, Deserialize)]
struct TreeCase {
    name: String,
    root_receipt: String,
    supplied_child_receipts: Vec<String>,
    config: FixtureTreeConfig,
    expected: ExpectedVerification,
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct FixtureTreeConfig {
    max_depth: usize,
    max_breadth: usize,
    require_parent_links: bool,
}

impl FixtureTreeConfig {
    fn to_receipt_config(self) -> ReceiptTreeConfig {
        ReceiptTreeConfig {
            max_depth: self.max_depth,
            max_breadth: self.max_breadth,
            require_parent_links: self.require_parent_links,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ExpectedVerification {
    valid: bool,
    findings: Vec<ExpectedFinding>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct ExpectedFinding {
    code: String,
    path: String,
}

#[test]
fn receipt_tree_fixture_oracle_matches_ordered_findings() -> Result<(), String> {
    let oracle: Oracle = serde_json::from_str(ORACLE).map_err(|error| error.to_string())?;

    for case in &oracle.cases {
        let root = oracle.receipt(&case.root_receipt)?;
        let children = oracle.child_receipts(&case.supplied_child_receipts)?;
        let resolver = FixtureResolver {
            children: &children,
        };

        let verification =
            verify_receipt_tree_with_resolver(root, &resolver, case.config.to_receipt_config());

        assert_eq!(
            verification.valid, case.expected.valid,
            "validity drifted for fixture case {}",
            case.name
        );
        assert_eq!(
            ordered_findings(&verification),
            case.expected.findings,
            "ordered findings drifted for fixture case {}",
            case.name
        );
    }

    Ok(())
}

impl Oracle {
    fn receipt(&self, name: &str) -> Result<&HarnessReceipt, String> {
        self.receipts
            .get(name)
            .ok_or_else(|| format!("receipt fixture {name} is missing"))
    }

    fn child_receipts(&self, names: &[String]) -> Result<Vec<HarnessReceipt>, String> {
        names
            .iter()
            .map(|name| self.receipt(name).cloned())
            .collect()
    }
}

struct FixtureResolver<'a> {
    children: &'a [HarnessReceipt],
}

impl ReceiptResolver for FixtureResolver<'_> {
    fn resolve_child<'a>(&'a self, reference: &Reference) -> ReceiptResolveResult<'a> {
        let Some(receipt_id) = referenced_receipt_id(reference) else {
            return ReceiptResolveResult::Malformed;
        };
        let mut matches = self
            .children
            .iter()
            .enumerate()
            .filter(|(_, child)| child.id == receipt_id);
        let Some((index, receipt)) = matches.next() else {
            return ReceiptResolveResult::Missing;
        };
        if matches.next().is_some() {
            return ReceiptResolveResult::Ambiguous;
        }
        ReceiptResolveResult::Found(ResolvedReceipt {
            path: format!("children[{index}]"),
            receipt,
        })
    }

    fn supplied_receipts<'a>(&'a self) -> Vec<ResolvedReceipt<'a>> {
        self.children
            .iter()
            .enumerate()
            .map(|(index, receipt)| ResolvedReceipt {
                path: format!("children[{index}]"),
                receipt,
            })
            .collect()
    }
}

fn referenced_receipt_id(reference: &Reference) -> Option<&str> {
    if reference.reference_type != ReferenceType::HarnessReceipt {
        return None;
    }
    reference
        .uri
        .strip_prefix("runx:harness_receipt:")
        .filter(|id| !id.is_empty())
        .or_else(|| {
            reference
                .uri
                .as_str()
                .split_once(':')
                .is_none()
                .then_some(reference.uri.as_str())
        })
}

fn ordered_findings(verification: &ReceiptVerification) -> Vec<ExpectedFinding> {
    verification
        .findings
        .iter()
        .map(|finding| ExpectedFinding {
            code: format!("{:?}", finding.code),
            path: finding.path.clone(),
        })
        .collect()
}
