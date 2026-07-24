"""Fail-closed scope for the active legacy-reference sweep."""

EXPECTED_CHECK_IDS = ("active-reference-sweep",)

ACTIVE_WORKTREE_INVENTORY_COMMAND = (
    "git",
    "ls-files",
    "--cached",
    "--others",
    "--exclude-standard",
    "-z",
)

GENERATED_EVIDENCE_PREFIX = "libs/openapi-codegen/external-contracts/evidence/"

HISTORICAL_ALLOWLIST = frozenset(
    {
        "libs/openapi-codegen/external-contracts/behavior/"
        "multi-language-openapi-client-generation-contract.md",
        "libs/openapi-codegen/external-contracts/ec.lock",
        "libs/openapi-codegen/external-contracts/ec-review.json",
    }
)
