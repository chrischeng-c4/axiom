"""Executable design for profile selection and diagnostic inclusion."""

from enum import Enum

__aw_artifact_id__ = "artifact:guard/design-scan-policy"


class PolicyProfile(Enum):
    BASELINE_STATIC = "guard-baseline-static/1"
    SECURITY_LINT = "guard-security-lint/1"
    STRICT = "guard-strict/1"


def included_rule(profile: PolicyProfile, category: str, rule: str) -> bool:
    if category == "security":
        return True
    return profile in {PolicyProfile.SECURITY_LINT, PolicyProfile.STRICT} and rule in {
        "DK002",
        "JS007",
        "JS008",
        "SQL-INJ",
        "TS102",
    }


def supported_languages() -> tuple[str, ...]:
    return (
        "python",
        "typescript",
        "rust",
        "javascript",
        "go",
        "html",
        "css",
        "dockerfile",
        "hcl",
        "yaml",
        "toml",
        "sql",
        "graphql",
    )
