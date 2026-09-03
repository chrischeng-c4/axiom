//! Private shutdown-report mapping for the Lumen serving binary.
//!
//! This module owns the observable decision that follows a terminal Raft host
//! shutdown report. The binary owns socket handles; this module keeps the
//! report-to-log and report-to-listener policy testable without a socket.

use raft_runtime::{HostShutdownReport, LeadershipHandoff, ShutdownPhase};

/// The peer-listener action must happen only after the terminal report is
/// emitted. The serving binary performs the actual close or abort.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PeerListenerAction {
    CloseAfterReport,
    AbortAfterReport,
}

/// Fields for Lumen's one terminal `raft_shutdown` JSON event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RaftShutdownEvent {
    pub(crate) shutdown_budget_ms: u64,
    pub(crate) proposal_admission: &'static str,
    pub(crate) handoff: &'static str,
    pub(crate) incomplete_phase: Option<&'static str>,
    pub(crate) peer_listener_close_safe: bool,
    pub(crate) listener_action: PeerListenerAction,
}

pub(crate) struct RaftShutdownCoordinator;

impl RaftShutdownCoordinator {
    pub(crate) fn event(report: &HostShutdownReport, shutdown_budget_ms: u64) -> RaftShutdownEvent {
        let listener_action = if report.peer_listener_close_safe {
            PeerListenerAction::CloseAfterReport
        } else {
            PeerListenerAction::AbortAfterReport
        };
        RaftShutdownEvent {
            shutdown_budget_ms,
            proposal_admission: "quiesced",
            handoff: handoff_name(&report.handoff),
            incomplete_phase: report.incomplete_phase.map(phase_name),
            peer_listener_close_safe: report.peer_listener_close_safe,
            listener_action,
        }
    }

    #[cfg_attr(feature = "raft-wal", allow(dead_code))]
    pub(crate) fn non_raft_action() -> Option<PeerListenerAction> {
        None
    }
}

fn handoff_name(handoff: &LeadershipHandoff) -> &'static str {
    match handoff {
        LeadershipHandoff::Transferred { .. } => "transferred",
        LeadershipHandoff::NotLeader => "not_leader",
        LeadershipHandoff::SoleVoter => "sole_voter",
        LeadershipHandoff::NoCaughtUpVoter { .. } => "no_caught_up_voter",
    }
}

fn phase_name(phase: ShutdownPhase) -> &'static str {
    match phase {
        ShutdownPhase::Quiesce => "quiesce",
        ShutdownPhase::LeadershipHandoff => "leadership_handoff",
        ShutdownPhase::BackgroundTasks => "background_tasks",
        ShutdownPhase::PeerRpcDrain => "peer_rpc_drain",
    }
}

#[cfg(test)]
mod tests {
    use raft_runtime::{HostShutdownReport, LeadershipHandoff, ShutdownCaller, ShutdownPhase};

    use super::*;

    fn report(
        handoff: LeadershipHandoff,
        incomplete_phase: Option<ShutdownPhase>,
        peer_listener_close_safe: bool,
    ) -> HostShutdownReport {
        HostShutdownReport {
            caller: ShutdownCaller::Executed,
            phases: Vec::new(),
            handoff,
            incomplete_phase,
            peer_listener_close_safe,
            storage_failure: None,
        }
    }

    #[test]
    fn safe_single_voter_report_closes_peer_listener_after_terminal_log() {
        let report = report(LeadershipHandoff::SoleVoter, None, true);

        let event = RaftShutdownCoordinator::event(&report, 1_000);

        assert_eq!(event.handoff, "sole_voter");
        assert_eq!(event.incomplete_phase, None);
        assert_eq!(event.listener_action, PeerListenerAction::CloseAfterReport);
        assert_eq!(event.shutdown_budget_ms, 1_000);
    }

    #[test]
    fn incomplete_quiesce_aborts_peer_listener_after_terminal_log() {
        let report = report(
            LeadershipHandoff::NotLeader,
            Some(ShutdownPhase::Quiesce),
            false,
        );

        let event = RaftShutdownCoordinator::event(&report, 0);

        assert_eq!(event.handoff, "not_leader");
        assert_eq!(event.incomplete_phase.as_deref(), Some("quiesce"));
        assert_eq!(event.listener_action, PeerListenerAction::AbortAfterReport);
    }

    #[test]
    fn non_raft_shutdown_has_no_peer_listener_decision() {
        assert_eq!(RaftShutdownCoordinator::non_raft_action(), None);
    }
}
