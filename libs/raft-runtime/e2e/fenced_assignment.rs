//! Public API conformance for committed executor ownership and fencing.

use raft_runtime::{AssignmentError, FenceToken, FencedAssignment};

#[test]
fn executor_cannot_act_before_assignment_commit() {
    let assignment = FencedAssignment::idle();
    assert_eq!(assignment.token(), None);
}

#[test]
fn expiry_and_reassignment_fence_the_previous_executor() {
    let mut assignment = FencedAssignment::idle();
    let first = assignment.assign(0, 100, 200).unwrap();

    assert!(matches!(
        assignment.assign(1, 201, 300),
        Err(AssignmentError::AlreadyAssigned(_))
    ));
    assignment.expire(200).unwrap();
    let second = assignment.assign(1, 200, 300).unwrap();

    assert_eq!(first, FenceToken { owner: 0, epoch: 1 });
    assert_eq!(second, FenceToken { owner: 1, epoch: 2 });
    assert!(matches!(
        assignment.validate(first, 201),
        Err(AssignmentError::StaleEpoch { .. })
    ));
    assert!(assignment.validate(second, 201).is_ok());
}

#[test]
fn proposer_supplied_time_keeps_replica_transitions_deterministic() {
    let mut replicas = [FencedAssignment::idle(), FencedAssignment::idle()];
    for state in &mut replicas {
        let token = state.assign(2, 1_000, 2_000).unwrap();
        state.renew(token, 1_500, 2_500).unwrap();
        state.release(token, 2_000).unwrap();
    }
    assert_eq!(replicas[0], replicas[1]);
}
