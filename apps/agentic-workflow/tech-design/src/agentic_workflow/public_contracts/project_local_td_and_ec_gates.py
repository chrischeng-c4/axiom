"""Public TD boundary for project-local Python TD and EC gates."""

__aw_artifact_id__ = "artifact:project-local-td-and-ec-gates/public-contract"
__aw_public_contract__ = True


def cache_safe_python_ec_source_discovery() -> str:
    return "EC discovery ignores undeclared cache binaries"


def python_only_ec_authoring_lifecycle() -> str:
    return "EC source is native Python without app-level Rust EC tests"


def project_local_td_and_ec_gates_operational_efficiency() -> str:
    return "project-local gates stay within their efficiency threshold"


def project_local_td_and_ec_gates_operational_stability() -> str:
    return "project-local gates remain stable under repeated execution"
