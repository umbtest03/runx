use runx_contracts::{HarnessReceipt, Reference, ReferenceType};
use runx_receipts::{
    ReceiptResolveResult, ReceiptResolver, ReceiptTreeConfig, ReceiptVerification, ResolvedReceipt,
    validate_receipt_tree_with_resolver, verify_receipt_tree_with_resolver,
};

#[derive(Clone, Debug, Default)]
pub struct RuntimeReceiptResolver {
    receipts: Vec<HarnessReceipt>,
}

impl RuntimeReceiptResolver {
    #[must_use]
    pub fn new(receipts: impl IntoIterator<Item = HarnessReceipt>) -> Self {
        Self {
            receipts: receipts.into_iter().collect(),
        }
    }

    #[must_use]
    pub fn receipts(&self) -> &[HarnessReceipt] {
        &self.receipts
    }
}

impl ReceiptResolver for RuntimeReceiptResolver {
    fn resolve_child<'a>(&'a self, reference: &Reference) -> ReceiptResolveResult<'a> {
        let Some(receipt_id) = referenced_receipt_id(reference) else {
            return ReceiptResolveResult::Malformed;
        };
        let mut matches = self
            .receipts
            .iter()
            .enumerate()
            .filter(|(_, receipt)| receipt.id == receipt_id);
        let Some((index, receipt)) = matches.next() else {
            return ReceiptResolveResult::Missing;
        };
        if matches.next().is_some() {
            return ReceiptResolveResult::Ambiguous;
        }
        ReceiptResolveResult::Found(ResolvedReceipt {
            path: runtime_receipt_path(index),
            receipt,
        })
    }

    fn supplied_receipts<'a>(&'a self) -> Vec<ResolvedReceipt<'a>> {
        self.receipts
            .iter()
            .enumerate()
            .map(|(index, receipt)| ResolvedReceipt {
                path: runtime_receipt_path(index),
                receipt,
            })
            .collect()
    }
}

pub fn validate_runtime_receipt_tree(
    root: &HarnessReceipt,
    receipts: impl IntoIterator<Item = HarnessReceipt>,
    config: ReceiptTreeConfig,
) -> Result<(), ReceiptVerification> {
    let resolver = RuntimeReceiptResolver::new(receipts);
    validate_receipt_tree_with_resolver(root, &resolver, config)
}

#[must_use]
pub fn verify_runtime_receipt_tree(
    root: &HarnessReceipt,
    receipts: impl IntoIterator<Item = HarnessReceipt>,
    config: ReceiptTreeConfig,
) -> ReceiptVerification {
    let resolver = RuntimeReceiptResolver::new(receipts);
    verify_receipt_tree_with_resolver(root, &resolver, config)
}

fn runtime_receipt_path(index: usize) -> String {
    format!("runtime_receipts[{index}]")
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
