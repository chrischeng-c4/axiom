"""Public TD boundary for Guard's policy and CLI profile."""

__aw_artifact_id__ = "artifact:guard/security-policy-profile"
__aw_public_contract__ = True


def baseline_static_policy() -> str:
    return "baseline-static maps security diagnostics into actionable findings"


def cli_module_registration() -> str:
    return "the public binary exposes scan, report, spec, and llm"


def security_lint_policy() -> str:
    return "security-lint adds security-impacting lint to baseline security"


def stable_policy_selection() -> str:
    return "equivalent scans preserve policy and actionable finding fields"
