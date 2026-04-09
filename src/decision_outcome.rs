//! Options and helpers for human-in-the-loop decision polling (mirrors Node `decision-outcome`).

use crate::types::NoxyHumanDecisionOutcome;

#[derive(Debug, Clone, Default)]
pub struct SendDecisionAndWaitOptions {
    pub initial_poll_interval_ms: Option<u64>,
    pub max_poll_interval_ms: Option<u64>,
    pub max_wait_ms: Option<u64>,
    pub backoff_multiplier: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct WaitForDecisionOutcomeOptions {
    pub decision_id: String,
    pub identity_id: String,
    pub initial_poll_interval_ms: Option<u64>,
    pub max_poll_interval_ms: Option<u64>,
    pub max_wait_ms: Option<u64>,
    pub backoff_multiplier: Option<f64>,
}

#[derive(Debug)]
pub struct WaitForDecisionOutcomeTimeoutError;

impl std::fmt::Display for WaitForDecisionOutcomeTimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("wait_for_decision_outcome exceeded max_wait_ms")
    }
}

impl std::error::Error for WaitForDecisionOutcomeTimeoutError {}

#[derive(Debug)]
pub struct SendDecisionAndWaitNoDecisionIdError;

impl std::fmt::Display for SendDecisionAndWaitNoDecisionIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("send_decision returned no decision_id to poll; check delivery statuses")
    }
}

impl std::error::Error for SendDecisionAndWaitNoDecisionIdError {}

pub fn is_terminal_human_outcome(outcome: NoxyHumanDecisionOutcome) -> bool {
    matches!(
        outcome,
        NoxyHumanDecisionOutcome::Approved
            | NoxyHumanDecisionOutcome::Rejected
            | NoxyHumanDecisionOutcome::Expired
    )
}
