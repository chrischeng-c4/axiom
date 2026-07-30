"""Public TD boundary for the capability control plane."""

__aw_artifact_id__ = "artifact:capability-control-plane/public-contract"
__aw_public_contract__ = True


def markdown_capability_schema() -> str:
    return "capability contracts use the canonical Markdown schema"


def missing_readme_initialization() -> str:
    return "missing capability documents have one initialization route"


def scoped_capability_verification() -> str:
    return "capability verification follows dependency closure"


def python_td_claim_linkage() -> str:
    return "Python TD artifact edges provide primary capability claim linkage"


def capability_control_plane_operational_efficiency() -> str:
    return "capability operations stay within their efficiency threshold"


def capability_control_plane_operational_stability() -> str:
    return "capability operations remain stable under repeated execution"
