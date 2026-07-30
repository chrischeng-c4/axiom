"""Public TD boundary for the Python-first artifact lifecycle."""

__aw_artifact_id__ = "artifact:aw-core-client-model-workitem-first-artifact-lifecycle/public-contract"
__aw_public_contract__ = True


def core_concept_model_and_invariants() -> str:
    return "phase-less work enters EC before TD"


def shared_artifact_producer_contract() -> str:
    return "artifact producers expose one typed continuation"


def aw_core_client_operational_efficiency() -> str:
    return "the artifact lifecycle stays within its efficiency threshold"


def aw_core_client_operational_stability() -> str:
    return "the artifact lifecycle remains stable under repeated execution"


def two_cell_ec_and_td_semantic_health() -> str:
    return "semantic health has exactly two cells"


def global_python_td_artifact_identity() -> str:
    return "TD artifact identity is globally unique"
