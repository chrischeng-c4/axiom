"Tech design for WI #3382: aw: separate CB materialization from controller-owned Git.\n\n@spec #3382"

from __future__ import annotations


__aw_artifact_id__ = "artifact:workflow-root-runner-td-cb-lifecycle-automation/separate-cb-materialization-from-controller-owned-git-wi-3382"
__aw_work_item__ = "3382"


def worker_safe_cb_materialization_candidate_purity() -> str:
    """R1: Provide deterministic CB candidate materialization without mutating Git or tracker state.

    The worker-safe 'aw cb materialize <slug>' command produces candidate files in the current
    clean linked worktree. It leaves Git HEAD, branch, index, remote refs, tracker issue body/state/labels,
    and lifecycle phase completely unchanged.
    """
    return (
        "worker-safe 'aw cb materialize' creates candidate files in clean linked worktree "
        "without mutating Git HEAD, branch, index, remote refs, tracker issue, or lifecycle phase"
    )


def controller_only_cb_publication_atomic_handshake() -> str:
    """R2: Provide distinct controller-owned publication operation that consumes verified candidate evidence.

    The controller-only 'aw cb publish <slug> --candidate-digest <sha256>' command validates current
    materialization evidence against the worktree, performing phase, event, and Git publication atomically,
    or refusing execution before any externally visible mutation occurs.
    """
    return (
        "controller-only 'aw cb publish --candidate-digest' validates candidate evidence before "
        "performing atomic Git commit/index advance, tracker projection update, and phase advance"
    )


def cb_td_admission_and_preflight_guards() -> str:
    """R3: Preserve TD/CB admission checks prior to candidate write or publication.

    Both materialize and publish operations enforce explicit path validation, anchor completeness,
    locked TD state, and clean linked worktree preflight guards before writing candidate files
    or publishing changes.
    """
    return (
        "materialize and publish operations require valid TD/CB path, required anchors, locked TD, "
        "and clean worktree state before candidate writing or controller publication"
    )


def typed_machine_readable_envelopes_and_refusals() -> str:
    """R4: Emit typed machine-readable envelopes for materialization, publication, and refusals.

    Envelopes explicitly distinguish candidate materialized ('action': 'materialized'), publication
    required, and typed fail-closed refusals (missing_candidate_digest, stale_candidate_digest,
    mismatched_candidate_digest) while ensuring workflow_complete remains False.
    """
    return (
        "envelopes distinguish action 'materialized', publication required, and typed refusals "
        "(missing_candidate_digest, stale_candidate_digest, mismatched_candidate_digest) with workflow_complete=False"
    )


def committed_terminal_cb_tracker_closure_authority() -> str:
    """R5: Preserve committed-CB-only tracker issue closure authority.

    A materialized candidate cannot close, publish, or repair a tracker issue by itself.
    Only a committed terminal CB lifecycle transition ('aw cb check') possesses the sole authority
    to close the tracker issue.
    """
    return (
        "materialized candidate does not alter tracker issue state; only a committed terminal CB transition "
        "('aw cb check') closes the tracker issue"
    )


def design_contract() -> str:
    """Express the executable design contract for WI #3382.

    R1: Deterministic CB materialization producing candidate files in clean linked worktree without mutating Git/tracker/phase.
    R2: Distinct controller-only publication validating evidence before atomic phase/event/Git publication.
    R3: TD/CB admission checks (path, anchor, lock, clean worktree) preserved before write/publish.
    R4: Typed envelopes distinguishing candidate materialized, publish commands, and typed refusal codes (missing/stale/mismatched).
    R5: Committed-CB-only tracker closure authority.
    """
    r1 = worker_safe_cb_materialization_candidate_purity()
    r2 = controller_only_cb_publication_atomic_handshake()
    r3 = cb_td_admission_and_preflight_guards()
    r4 = typed_machine_readable_envelopes_and_refusals()
    r5 = committed_terminal_cb_tracker_closure_authority()

    assert "materialize" in r1 and "without mutating" in r1
    assert "publish" in r2 and "atomic" in r2
    assert "clean worktree" in r3
    assert "envelopes" in r4 and "refusal" in r4
    assert "closes" in r5

    return "ok"
