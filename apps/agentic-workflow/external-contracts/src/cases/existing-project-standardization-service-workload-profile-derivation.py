"""Black-box contract for the stateful-workload trait-to-profile derivation (#3310)."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import project_fixture, run_aw

CASE_ID = "existing-project-standardization-service-workload-profile-derivation"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "service-workload-profile-derivation"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case existing-project-standardization-service-workload-profile-derivation"
)
ASSERTIONS = (
    "aw meta sync --repository-product, run against a CONTRIBUTING.md whose "
    "aw:trait-table block holds a stale placeholder row, replaces that block "
    "with the live-rendered trait table and removes the placeholder, proving "
    "the repo trait-table producer is a real splice-on-existing-markers "
    "projection rather than a permanently-inert or hand-maintained block",
    "the freshly-projected stateful_storage row derives the "
    "stateful-service-workload baseline capability and links the "
    "'Service workload profiles' StatefulSet/Deployment CONTRIBUTING.md "
    "section on that same row, proving the StatefulSet service workload "
    "profile is genuinely derived from the trait registry rather than "
    "independently hand-typed prose that could drift from it",
)

_CONTRIBUTING_FIXTURE = """# CONTRIBUTING (fixture)

<!-- aw:trait-table:start -->
| Trait | Derives | Enforces | About |
|---|---|---|---|
| `stale_placeholder_trait` | `stale-placeholder-cap` | — | stale row that must be replaced |
<!-- aw:trait-table:end -->
"""


def verify() -> list[str]:
    with project_fixture() as root:
        (root / "CONTRIBUTING.md").write_text(_CONTRIBUTING_FIXTURE, encoding="utf-8")

        run_aw(root, "meta", "sync", "--repository-product")

        contributing = (root / "CONTRIBUTING.md").read_text(encoding="utf-8")
        start = contributing.index("<!-- aw:trait-table:start -->")
        end = contributing.index("<!-- aw:trait-table:end -->")
        block = contributing[start:end]

        assert "stale_placeholder_trait" not in block, block
        assert "stale-placeholder-cap" not in block, block

        stateful_row = next(
            (line for line in block.splitlines() if "`stateful_storage`" in line), None
        )
        assert stateful_row is not None, block
        assert "`stateful-service-workload`" in stateful_row, stateful_row
        assert "Service workload profiles" in stateful_row, stateful_row
        assert "StatefulSet" in stateful_row, stateful_row

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
