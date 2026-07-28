"""Public TD boundary for the workflow root runner."""

__aw_artifact_id__ = "artifact:workflow-root-runner/public-contract"
__aw_public_contract__ = True


def fail_closed_coordination_event_validation() -> str:
    return "invalid coordination events fail closed"


def aw_only_completion_and_decision_authority() -> str:
    return "only durable AW state advances completion or decisions"


def versioned_client_independent_coordination_contract() -> str:
    return "coordination documents share one versioned public schema"


def self_hosting_root_runner_policy() -> str:
    return "self-hosting uses Python-first roots and bounded direct repair"


def parent_rollup_routing() -> str:
    return "child completion rolls up until the parent root is terminal"
