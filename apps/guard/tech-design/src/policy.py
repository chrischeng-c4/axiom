"""Guard security policy contract and executable selection rules."""

from enum import Enum

__aw_artifact_id__ = "artifact:guard/security-policy-profile"
__aw_public_contract__ = True


class PolicyProfile(Enum):
    BASELINE_STATIC = "guard-baseline-static/1"
    SECURITY_LINT = "guard-security-lint/1"
    STRICT = "guard-strict/1"


class PolicyDesign:
    """Fail-closed profile behavior implemented by Guard scanning."""

    SECURITY_IMPACTING_LINT = frozenset(
        {"DK002", "JS007", "JS008", "SQL-INJ", "TS102"}
    )

    @classmethod
    def included_rule(
        cls,
        profile: PolicyProfile,
        category: str,
        rule: str,
    ) -> bool:
        if category == "security":
            return True
        return (
            profile in {PolicyProfile.SECURITY_LINT, PolicyProfile.STRICT}
            and rule in cls.SECURITY_IMPACTING_LINT
        )


def baseline_static_policy() -> str:
    return "baseline-static maps security diagnostics into actionable findings"


def standalone_cli_distribution() -> str:
    return "the standalone guard package builds a binary exposing scan, report, spec, and llm"


def security_lint_policy() -> str:
    return "security-lint adds security-impacting lint to baseline security"


def stable_policy_selection() -> str:
    return "equivalent scans preserve policy and actionable finding fields"
