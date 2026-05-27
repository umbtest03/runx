use super::super::types::{
    GraphStatus, GraphStepStatus, SequentialGraphState, SequentialGraphStepDefinition,
    SequentialGraphStepState,
};

#[must_use]
pub fn create_sequential_graph_state(
    graph_id: impl Into<String>,
    steps: &[SequentialGraphStepDefinition],
) -> SequentialGraphState {
    SequentialGraphState {
        graph_id: graph_id.into(),
        status: GraphStatus::Pending,
        steps: steps
            .iter()
            .map(|step| SequentialGraphStepState {
                step_id: step.id.clone(),
                status: GraphStepStatus::Pending,
                attempts: 0,
                started_at: None,
                completed_at: None,
                receipt_id: None,
                outputs: None,
                error: None,
            })
            .collect(),
    }
}
