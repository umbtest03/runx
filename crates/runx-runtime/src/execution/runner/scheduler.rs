use std::collections::BTreeMap;
use std::path::Path;

use runx_parser::{ExecutionGraph, GraphStep};

use super::super::graph::{find_step, load_step_skill};
use super::RUNX_MAX_FANOUT_CONCURRENCY_ENV;
use crate::RuntimeError;
use crate::adapter::{FanoutExecutionMode, SkillAdapter};

const DEFAULT_MAX_FANOUT_CONCURRENCY: usize = 1;
const HARD_MAX_FANOUT_CONCURRENCY: usize = 64;

pub(super) struct FanoutScheduler {
    max_concurrency: usize,
}

pub(super) enum FanoutSchedule<'a> {
    Serial(Vec<ScheduledFanoutStep<'a>>),
    Parallel(ParallelFanoutSchedule<'a>),
}

pub(super) struct ParallelFanoutSchedule<'a> {
    pub(super) steps: Vec<ScheduledFanoutStep<'a>>,
    pub(super) max_concurrency: usize,
}

#[derive(Clone, Copy)]
pub(super) struct ScheduledFanoutStep<'a> {
    pub(super) step_id: &'a str,
    pub(super) attempt: u32,
}

impl FanoutScheduler {
    pub(super) fn from_env(env: &BTreeMap<String, String>) -> Self {
        Self {
            max_concurrency: configured_max_concurrency(env),
        }
    }

    pub(super) fn schedule<'a, A>(
        &self,
        adapter: &A,
        graph_dir: &Path,
        graph: &ExecutionGraph,
        step_ids: &'a [String],
        attempts: &'a BTreeMap<String, u32>,
    ) -> Result<FanoutSchedule<'a>, RuntimeError>
    where
        A: SkillAdapter,
    {
        let steps = scheduled_steps(step_ids, attempts);
        if self.max_concurrency <= 1 || steps.len() <= 1 {
            return Ok(FanoutSchedule::Serial(steps));
        }
        if !self.can_run_parallel(adapter, graph_dir, graph, &steps)? {
            return Ok(FanoutSchedule::Serial(steps));
        }
        Ok(FanoutSchedule::Parallel(ParallelFanoutSchedule {
            steps,
            max_concurrency: self.max_concurrency,
        }))
    }

    fn can_run_parallel<A>(
        &self,
        adapter: &A,
        graph_dir: &Path,
        graph: &ExecutionGraph,
        steps: &[ScheduledFanoutStep<'_>],
    ) -> Result<bool, RuntimeError>
    where
        A: SkillAdapter,
    {
        for scheduled in steps {
            let step = find_step(graph, scheduled.step_id)?;
            if !parallel_safe_step_shape(step) {
                return Ok(false);
            }
            let Ok(skill) = load_step_skill(graph_dir, step) else {
                return Ok(false);
            };
            if adapter.fanout_execution_mode(&skill.source) != FanoutExecutionMode::IsolatedParallel
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

fn scheduled_steps<'a>(
    step_ids: &'a [String],
    attempts: &'a BTreeMap<String, u32>,
) -> Vec<ScheduledFanoutStep<'a>> {
    step_ids
        .iter()
        .map(|step_id| ScheduledFanoutStep {
            step_id,
            attempt: attempts.get(step_id).copied().unwrap_or(1),
        })
        .collect()
}

fn parallel_safe_step_shape(step: &GraphStep) -> bool {
    step.run.is_none()
        && step.tool.is_none()
        && !step.mutating
        && !has_payment_authority_inputs(step)
}

fn has_payment_authority_inputs(step: &GraphStep) -> bool {
    step.inputs.keys().any(|key| is_payment_authority_key(key))
        || step
            .context_edges
            .iter()
            .any(|edge| is_payment_authority_key(&edge.input))
}

fn is_payment_authority_key(key: &str) -> bool {
    matches!(
        key,
        "reserved_payment_authority" | "spend_capability_ref" | "payment_challenge"
    )
}

fn configured_max_concurrency(env: &BTreeMap<String, String>) -> usize {
    env.get(RUNX_MAX_FANOUT_CONCURRENCY_ENV)
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_MAX_FANOUT_CONCURRENCY)
        .min(HARD_MAX_FANOUT_CONCURRENCY)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_serial_fanout() {
        assert_eq!(
            configured_max_concurrency(&BTreeMap::new()),
            DEFAULT_MAX_FANOUT_CONCURRENCY
        );
    }

    #[test]
    fn clamps_configured_fanout_concurrency() {
        let mut env = BTreeMap::new();
        env.insert(
            RUNX_MAX_FANOUT_CONCURRENCY_ENV.to_owned(),
            "100000".to_owned(),
        );
        assert_eq!(
            configured_max_concurrency(&env),
            HARD_MAX_FANOUT_CONCURRENCY
        );
    }
}
