// rust-style-allow: large-file -- tree traversal, resolver outcomes, and the
// adversarial unit matrix stay together until the resolver contract stabilizes.
use std::collections::{BTreeMap, BTreeSet};

use runx_contracts::{HarnessReceipt, Reference, ReferenceType};

use crate::{
    ReceiptFinding, ReceiptFindingCode, ReceiptVerification, validate_harness_receipt,
    verify_harness_receipt,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReceiptTreeConfig {
    pub max_depth: usize,
    pub max_breadth: usize,
    pub require_parent_links: bool,
}

impl Default for ReceiptTreeConfig {
    fn default() -> Self {
        Self {
            max_depth: 64,
            max_breadth: 1024,
            require_parent_links: false,
        }
    }
}

pub trait ReceiptResolver {
    fn resolve_child<'a>(&'a self, reference: &Reference) -> ReceiptResolveResult<'a>;
    fn supplied_receipts<'a>(&'a self) -> Vec<ResolvedReceipt<'a>>;
}

#[derive(Clone, Debug)]
pub struct ResolvedReceipt<'a> {
    pub path: String,
    pub receipt: &'a HarnessReceipt,
}

#[derive(Clone, Debug)]
pub enum ReceiptResolveResult<'a> {
    Found(ResolvedReceipt<'a>),
    Missing,
    Malformed,
    Ambiguous,
}

pub fn validate_receipt_tree(
    root: &HarnessReceipt,
    children: &[HarnessReceipt],
) -> Result<(), ReceiptVerification> {
    let resolver = SliceReceiptResolver { children };
    validate_receipt_tree_with_resolver(root, &resolver, ReceiptTreeConfig::default())
}

#[must_use]
pub fn verify_receipt_tree(
    root: &HarnessReceipt,
    children: &[HarnessReceipt],
) -> ReceiptVerification {
    let resolver = SliceReceiptResolver { children };
    verify_receipt_tree_with_resolver(root, &resolver, ReceiptTreeConfig::default())
}

pub fn validate_receipt_tree_with_resolver(
    root: &HarnessReceipt,
    resolver: &impl ReceiptResolver,
    config: ReceiptTreeConfig,
) -> Result<(), ReceiptVerification> {
    let verification = verify_receipt_tree_with_resolver(root, resolver, config);
    if verification.valid {
        Ok(())
    } else {
        Err(verification)
    }
}

#[must_use]
pub fn verify_receipt_tree_with_resolver(
    root: &HarnessReceipt,
    resolver: &impl ReceiptResolver,
    config: ReceiptTreeConfig,
) -> ReceiptVerification {
    let mut findings = verify_harness_receipt(root).findings;
    let supplied = resolver.supplied_receipts();
    findings.extend(duplicate_child_findings(&supplied));
    findings.extend(child_receipt_findings(&supplied));
    let mut traversal = TreeTraversal {
        resolver,
        config,
        visiting: BTreeSet::new(),
        reached: BTreeSet::new(),
    };
    findings.extend(traversal.subtree_findings("harness", root, 0));
    findings.extend(orphan_child_findings(&supplied, &traversal.reached));
    ReceiptVerification::from_findings(findings)
}

struct SliceReceiptResolver<'a> {
    children: &'a [HarnessReceipt],
}

impl ReceiptResolver for SliceReceiptResolver<'_> {
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

fn duplicate_child_findings(children: &[ResolvedReceipt<'_>]) -> Vec<ReceiptFinding> {
    let mut seen = BTreeMap::new();
    children
        .iter()
        .filter_map(|child| {
            if seen
                .insert(child.receipt.id.as_str(), child.path.as_str())
                .is_some()
            {
                Some(ReceiptFinding {
                    code: ReceiptFindingCode::DuplicateChildReceipt,
                    path: format!("{}.id", child.path),
                    message: "child receipt ids must be unique".to_owned(),
                })
            } else {
                None
            }
        })
        .collect()
}

fn child_receipt_findings(children: &[ResolvedReceipt<'_>]) -> Vec<ReceiptFinding> {
    children
        .iter()
        .flat_map(|child| {
            validate_harness_receipt(child.receipt)
                .err()
                .map_or_else(Vec::new, |verification| {
                    verification
                        .findings
                        .into_iter()
                        .map(|finding| child_finding(&child.path, finding))
                        .collect()
                })
        })
        .collect()
}

struct TreeTraversal<'a, R: ReceiptResolver> {
    resolver: &'a R,
    config: ReceiptTreeConfig,
    visiting: BTreeSet<String>,
    reached: BTreeSet<String>,
}

impl<R: ReceiptResolver> TreeTraversal<'_, R> {
    fn subtree_findings(
        &mut self,
        path: &str,
        receipt: &HarnessReceipt,
        depth: usize,
    ) -> Vec<ReceiptFinding> {
        if !self.visiting.insert(receipt.id.clone()) {
            return vec![ReceiptFinding {
                code: ReceiptFindingCode::ChildReceiptCycle,
                path: format!("{path}.id"),
                message: "child harness receipt refs must not form cycles".to_owned(),
            }];
        }

        let mut findings = Vec::new();
        let child_refs = &receipt.harness.child_harness_receipt_refs;
        if child_refs.len() > self.config.max_breadth {
            findings.push(ReceiptFinding {
                code: ReceiptFindingCode::ChildReceiptBreadthLimit,
                path: format!("{path}.child_harness_receipt_refs"),
                message: "child harness receipt refs exceed configured breadth limit".to_owned(),
            });
        }

        let child_findings = receipt
            .harness
            .child_harness_receipt_refs
            .iter()
            .take(self.config.max_breadth)
            .enumerate()
            .flat_map(|(index, reference)| {
                self.child_ref_findings(
                    &format!("{path}.child_harness_receipt_refs[{index}]"),
                    receipt,
                    reference,
                    depth,
                )
            })
            .collect::<Vec<_>>();
        findings.extend(child_findings);
        self.visiting.remove(receipt.id.as_str());
        findings
    }

    fn child_ref_findings(
        &mut self,
        path: &str,
        parent: &HarnessReceipt,
        reference: &Reference,
        depth: usize,
    ) -> Vec<ReceiptFinding> {
        if reference.reference_type != ReferenceType::HarnessReceipt {
            return vec![malformed_child_ref(path)];
        };
        let next_depth = depth.saturating_add(1);
        if next_depth > self.config.max_depth {
            return vec![ReceiptFinding {
                code: ReceiptFindingCode::ChildReceiptDepthLimit,
                path: path.to_owned(),
                message: "child harness receipt refs exceed configured depth limit".to_owned(),
            }];
        };
        let resolved = match self.resolver.resolve_child(reference) {
            ReceiptResolveResult::Found(resolved) => resolved,
            ReceiptResolveResult::Missing => return vec![missing_child(path)],
            ReceiptResolveResult::Malformed => return vec![malformed_child_ref(path)],
            ReceiptResolveResult::Ambiguous => return vec![ambiguous_child(path)],
        };
        let child = resolved.receipt;
        if self.visiting.contains(child.id.as_str()) {
            return vec![ReceiptFinding {
                code: ReceiptFindingCode::ChildReceiptCycle,
                path: path.to_owned(),
                message: "child harness receipt refs must not point to an ancestor".to_owned(),
            }];
        }
        if self.reached.contains(child.id.as_str()) {
            return Vec::new();
        }
        let child_path = format!("{}.harness", resolved.path);
        let mut findings = parent_link_findings(path, parent, child, self.config);
        findings.extend(self.subtree_findings(&child_path, child, next_depth));
        self.reached.insert(child.id.clone());
        findings
    }
}

fn orphan_child_findings(
    children: &[ResolvedReceipt<'_>],
    reached: &BTreeSet<String>,
) -> Vec<ReceiptFinding> {
    children
        .iter()
        .filter(|child| !reached.contains(child.receipt.id.as_str()))
        .map(|child| ReceiptFinding {
            code: ReceiptFindingCode::OrphanChildReceipt,
            path: format!("{}.id", child.path),
            message: "supplied child receipts must be reachable from the root receipt".to_owned(),
        })
        .collect()
}

fn missing_child(path: &str) -> ReceiptFinding {
    ReceiptFinding {
        code: ReceiptFindingCode::ChildReceiptMissing,
        path: path.to_owned(),
        message: "child harness receipt ref must resolve to a supplied child receipt".to_owned(),
    }
}

fn malformed_child_ref(path: &str) -> ReceiptFinding {
    ReceiptFinding {
        code: ReceiptFindingCode::ChildReceiptRefMalformed,
        path: path.to_owned(),
        message: "child harness receipt ref must be a typed runx harness receipt URI or exact id"
            .to_owned(),
    }
}

fn ambiguous_child(path: &str) -> ReceiptFinding {
    ReceiptFinding {
        code: ReceiptFindingCode::ChildReceiptAmbiguous,
        path: path.to_owned(),
        message: "child harness receipt ref resolved to multiple supplied receipts".to_owned(),
    }
}

fn parent_link_findings(
    path: &str,
    parent: &HarnessReceipt,
    child: &HarnessReceipt,
    config: ReceiptTreeConfig,
) -> Vec<ReceiptFinding> {
    match &child.harness.parent_harness_ref {
        Some(parent_ref) if parent_ref == &parent.harness.harness_ref => Vec::new(),
        Some(_) => vec![ReceiptFinding {
            code: ReceiptFindingCode::ChildReceiptParentMismatch,
            path: format!("{path}.parent_harness_ref"),
            message: "child harness parent ref must match the parent harness ref".to_owned(),
        }],
        None if config.require_parent_links => vec![ReceiptFinding {
            code: ReceiptFindingCode::ChildReceiptParentMismatch,
            path: format!("{path}.parent_harness_ref"),
            message: "strict tree verification requires child harness parent refs".to_owned(),
        }],
        None => Vec::new(),
    }
}

fn child_finding(path: &str, finding: ReceiptFinding) -> ReceiptFinding {
    ReceiptFinding {
        path: format!("{path}.{}", finding.path),
        ..finding
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

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::{
        ReceiptResolveResult, ReceiptResolver, ReceiptTreeConfig, ResolvedReceipt,
        validate_receipt_tree_with_resolver, verify_receipt_tree,
        verify_receipt_tree_with_resolver,
    };
    use crate::{ReceiptFindingCode, ReceiptVerification};
    use runx_contracts::{HarnessReceipt, Reference, ReferenceType};

    const SUCCESS_RECEIPT: &str =
        include_str!("../../../fixtures/contracts/harness-spine/harness-receipt-success.json");
    const ABNORMAL_RECEIPT: &str =
        include_str!("../../../fixtures/contracts/harness-spine/harness-receipt-abnormal.json");

    #[derive(Debug, Deserialize)]
    struct Fixture {
        expected: HarnessReceipt,
    }

    #[test]
    fn slice_adapter_accepts_exact_id_and_typed_uri() -> Result<(), serde_json::Error> {
        let mut root = fixture(SUCCESS_RECEIPT)?;
        let child = child("hrn_rcpt_child_1")?;

        root.harness.child_harness_receipt_refs[0].uri = "hrn_rcpt_child_1".to_owned();
        assert!(verify_receipt_tree(&root, std::slice::from_ref(&child)).valid);

        root.harness.child_harness_receipt_refs[0].uri =
            "runx:harness_receipt:hrn_rcpt_child_1".to_owned();
        assert!(verify_receipt_tree(&root, &[child]).valid);
        Ok(())
    }

    #[test]
    fn malformed_and_wrong_namespace_refs_are_stable_findings() -> Result<(), serde_json::Error> {
        let mut root = fixture(SUCCESS_RECEIPT)?;
        let child = child("hrn_rcpt_child_1")?;

        root.harness.child_harness_receipt_refs[0].uri = "runx:receipt:hrn_rcpt_child_1".to_owned();
        let verification = verify_receipt_tree(&root, std::slice::from_ref(&child));
        assert_finding(
            &verification,
            ReceiptFindingCode::ChildReceiptRefMalformed,
            "harness.child_harness_receipt_refs[0]",
        );

        root.harness.child_harness_receipt_refs[0].uri = ":hrn_rcpt_child_1".to_owned();
        let verification = verify_receipt_tree(&root, &[child]);
        assert_finding(
            &verification,
            ReceiptFindingCode::ChildReceiptRefMalformed,
            "harness.child_harness_receipt_refs[0]",
        );
        Ok(())
    }

    #[test]
    fn suffix_only_refs_do_not_resolve_by_suffix() -> Result<(), serde_json::Error> {
        let mut root = fixture(SUCCESS_RECEIPT)?;
        root.harness.child_harness_receipt_refs[0].uri = "child_1".to_owned();
        let child = child("hrn_rcpt_child_1")?;

        let verification = verify_receipt_tree(&root, &[child]);

        assert_finding(
            &verification,
            ReceiptFindingCode::ChildReceiptMissing,
            "harness.child_harness_receipt_refs[0]",
        );
        Ok(())
    }

    #[test]
    fn duplicate_ids_make_slice_resolution_ambiguous() -> Result<(), serde_json::Error> {
        let root = fixture(SUCCESS_RECEIPT)?;
        let first = child("hrn_rcpt_child_1")?;
        let second = child("hrn_rcpt_child_1")?;

        let verification = verify_receipt_tree(&root, &[first, second]);

        assert_finding(
            &verification,
            ReceiptFindingCode::DuplicateChildReceipt,
            "children[1].id",
        );
        assert_finding(
            &verification,
            ReceiptFindingCode::ChildReceiptAmbiguous,
            "harness.child_harness_receipt_refs[0]",
        );
        Ok(())
    }

    #[test]
    fn resolver_ambiguous_result_is_a_stable_finding() -> Result<(), serde_json::Error> {
        let root = fixture(SUCCESS_RECEIPT)?;

        let verification = verify_receipt_tree_with_resolver(
            &root,
            &AmbiguousResolver,
            ReceiptTreeConfig::default(),
        );

        assert_finding(
            &verification,
            ReceiptFindingCode::ChildReceiptAmbiguous,
            "harness.child_harness_receipt_refs[0]",
        );
        Ok(())
    }

    #[test]
    fn strict_mode_rejects_mismatched_parent_link() -> Result<(), serde_json::Error> {
        let root = fixture(SUCCESS_RECEIPT)?;
        let mut child = child("hrn_rcpt_child_1")?;
        child.harness.parent_harness_ref = Some(reference(ReferenceType::Harness, "other"));

        let verification = verify_receipt_tree_with_resolver(
            &root,
            &super::SliceReceiptResolver {
                children: std::slice::from_ref(&child),
            },
            ReceiptTreeConfig {
                require_parent_links: true,
                ..ReceiptTreeConfig::default()
            },
        );

        assert_finding(
            &verification,
            ReceiptFindingCode::ChildReceiptParentMismatch,
            "harness.child_harness_receipt_refs[0].parent_harness_ref",
        );
        Ok(())
    }

    #[test]
    fn strict_mode_requires_present_parent_link() -> Result<(), serde_json::Error> {
        let root = fixture(SUCCESS_RECEIPT)?;
        let child = child("hrn_rcpt_child_1")?;

        let verification = verify_receipt_tree_with_resolver(
            &root,
            &super::SliceReceiptResolver {
                children: std::slice::from_ref(&child),
            },
            ReceiptTreeConfig {
                require_parent_links: true,
                ..ReceiptTreeConfig::default()
            },
        );

        assert_finding(
            &verification,
            ReceiptFindingCode::ChildReceiptParentMismatch,
            "harness.child_harness_receipt_refs[0].parent_harness_ref",
        );
        Ok(())
    }

    #[test]
    fn depth_limit_blocks_hostile_nested_tree() -> Result<(), serde_json::Error> {
        let root = fixture(SUCCESS_RECEIPT)?;
        let mut child_receipt = child("hrn_rcpt_child_1")?;
        child_receipt
            .harness
            .child_harness_receipt_refs
            .push(reference(ReferenceType::HarnessReceipt, "grandchild"));
        let grandchild = child("grandchild")?;

        let verification = verify_receipt_tree_with_resolver(
            &root,
            &super::SliceReceiptResolver {
                children: &[child_receipt, grandchild],
            },
            ReceiptTreeConfig {
                max_depth: 1,
                ..ReceiptTreeConfig::default()
            },
        );

        assert_finding(
            &verification,
            ReceiptFindingCode::ChildReceiptDepthLimit,
            "children[0].harness.child_harness_receipt_refs[0]",
        );
        Ok(())
    }

    #[test]
    fn breadth_limit_blocks_hostile_fanout() -> Result<(), serde_json::Error> {
        let mut root = fixture(SUCCESS_RECEIPT)?;
        root.harness
            .child_harness_receipt_refs
            .push(reference(ReferenceType::HarnessReceipt, "second"));
        let first = child("hrn_rcpt_child_1")?;
        let second = child("second")?;

        let verification = verify_receipt_tree_with_resolver(
            &root,
            &super::SliceReceiptResolver {
                children: &[first, second],
            },
            ReceiptTreeConfig {
                max_breadth: 1,
                ..ReceiptTreeConfig::default()
            },
        );

        assert_finding(
            &verification,
            ReceiptFindingCode::ChildReceiptBreadthLimit,
            "harness.child_harness_receipt_refs",
        );
        Ok(())
    }

    #[test]
    fn positive_nested_tree_verifies() -> Result<(), serde_json::Error> {
        let root = fixture(SUCCESS_RECEIPT)?;
        let mut child_receipt = child("hrn_rcpt_child_1")?;
        child_receipt
            .harness
            .child_harness_receipt_refs
            .push(reference(ReferenceType::HarnessReceipt, "grandchild"));
        let grandchild = child("grandchild")?;

        assert!(verify_receipt_tree(&root, &[child_receipt, grandchild]).valid);
        Ok(())
    }

    #[test]
    fn positive_fanout_tree_verifies() -> Result<(), serde_json::Error> {
        let mut root = fixture(SUCCESS_RECEIPT)?;
        root.harness
            .child_harness_receipt_refs
            .push(reference(ReferenceType::HarnessReceipt, "second"));
        let first = child("hrn_rcpt_child_1")?;
        let second = child("second")?;

        assert!(verify_receipt_tree(&root, &[first, second]).valid);
        Ok(())
    }

    #[test]
    fn strict_parent_links_can_verify_cleanly() -> Result<(), serde_json::Error> {
        let root = fixture(SUCCESS_RECEIPT)?;
        let mut child = child("hrn_rcpt_child_1")?;
        child.harness.parent_harness_ref = Some(root.harness.harness_ref.clone());

        assert!(
            validate_receipt_tree_with_resolver(
                &root,
                &super::SliceReceiptResolver {
                    children: std::slice::from_ref(&child),
                },
                ReceiptTreeConfig {
                    require_parent_links: true,
                    ..ReceiptTreeConfig::default()
                },
            )
            .is_ok()
        );
        Ok(())
    }

    fn fixture(json: &str) -> Result<HarnessReceipt, serde_json::Error> {
        serde_json::from_str::<Fixture>(json).map(|fixture| fixture.expected)
    }

    fn child(id: &str) -> Result<HarnessReceipt, serde_json::Error> {
        let mut receipt = fixture(ABNORMAL_RECEIPT)?;
        receipt.id = id.to_owned();
        Ok(receipt)
    }

    fn reference(reference_type: ReferenceType, id: &str) -> Reference {
        Reference {
            uri: format!("runx:{}:{id}", reference_type_name(&reference_type)),
            reference_type,
            provider: None,
            locator: None,
            label: None,
            observed_at: None,
        }
    }

    fn reference_type_name(reference_type: &ReferenceType) -> &'static str {
        match reference_type {
            ReferenceType::HarnessReceipt => "harness_receipt",
            ReferenceType::Harness => "harness",
            _ => "reference",
        }
    }

    fn assert_finding(verification: &ReceiptVerification, code: ReceiptFindingCode, path: &str) {
        assert!(
            verification
                .findings
                .iter()
                .any(|finding| finding.code == code && finding.path == path),
            "expected finding {code:?} at {path}; got {:?}",
            verification.findings
        );
    }

    struct AmbiguousResolver;

    impl ReceiptResolver for AmbiguousResolver {
        fn resolve_child<'a>(&'a self, _reference: &Reference) -> ReceiptResolveResult<'a> {
            ReceiptResolveResult::Ambiguous
        }

        fn supplied_receipts<'a>(&'a self) -> Vec<ResolvedReceipt<'a>> {
            Vec::new()
        }
    }
}
