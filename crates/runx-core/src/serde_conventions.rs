//! Serde conventions for the Rust parity kernel.
//!
//! Public structs serialize with camelCase field names to match TypeScript
//! fixtures. Tagged unions use the same discriminator field as TypeScript,
//! usually `type` for state-machine events and plans. Optional fields are
//! omitted when absent. Serialized maps use deterministic key order.

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::state_machine::{FanoutSyncDecision, FanoutSyncStrategy, SequentialGraphPlan};

    #[test]
    fn state_machine_plan_uses_type_tag_and_camel_case_fields() -> Result<(), serde_json::Error> {
        let plan = SequentialGraphPlan::RunFanout {
            group_id: "advisors".to_owned(),
            step_ids: vec!["market".to_owned(), "risk".to_owned()],
            attempts: BTreeMap::from([("market".to_owned(), 1), ("risk".to_owned(), 1)]),
            context_from: BTreeMap::from([
                ("market".to_owned(), Vec::new()),
                ("risk".to_owned(), Vec::new()),
            ]),
        };

        let json = serde_json::to_string(&plan)?;

        assert_eq!(
            json,
            r#"{"type":"run_fanout","groupId":"advisors","stepIds":["market","risk"],"attempts":{"market":1,"risk":1},"contextFrom":{"market":[],"risk":[]}}"#,
        );
        Ok(())
    }

    #[test]
    fn optional_fields_are_omitted_when_absent() -> Result<(), serde_json::Error> {
        let decision = FanoutSyncDecision {
            group_id: "advisors".to_owned(),
            decision: crate::state_machine::FanoutSyncOutcome::Proceed,
            strategy: FanoutSyncStrategy::All,
            rule_fired: "all.min_success".to_owned(),
            reason: "2/2 branches succeeded; required 2".to_owned(),
            branch_count: 2,
            success_count: 2,
            failure_count: 0,
            required_successes: 2,
            gate: None,
        };

        let value = serde_json::to_value(decision)?;

        assert!(value.get("gate").is_none());
        Ok(())
    }
}
