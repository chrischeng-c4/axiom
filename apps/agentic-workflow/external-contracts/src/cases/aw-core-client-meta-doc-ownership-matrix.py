"""Black-box contract for the META-doc ownership matrix (#3306).

Drives real `aw meta init`/`check`/`sync` against a fixture project and
proves one serializable ownership matrix -- not three separately hand-coded
checks -- drives placement (a forbidden project-local AGENTS.md), the root
CAPABILITIES.md allowlist (rejected on a non-product repository root), and
matrix-owned required headings, then proves `aw meta sync` mechanically
repairs the marker-owned content violation while leaving both placement
violations for a human/agent to resolve rather than silently relocating or
deleting a misplaced file.
"""

from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import final_json, run_aw

CASE_ID = "aw-core-client-meta-doc-ownership-matrix"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "meta-doc-ownership-matrix"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-meta-doc-ownership-matrix"
)
ASSERTIONS = (
    "a freshly aw meta init-scaffolded repository/project layout carries none "
    "of the placement, root-allowlist, or heading findings, but adding a "
    "project-local apps/tool/AGENTS.md, a root-layer CAPABILITIES.md on a "
    "repository root not declared a product, and stripping the matrix-owned "
    "'### Non-Core Features' heading from the project CAPABILITIES.md and "
    "then running the identical aw meta check surfaces exactly those three "
    "diagnostics -- project_agent_doc_forbidden, "
    "root_capabilities_requires_product, and meta_doc_section_missing naming "
    "the exact missing heading -- proving one ownership matrix drives "
    "placement, root allowlist, and required-heading diagnostics together",
    "aw meta sync against that same violated layout mechanically restores "
    "the matrix-owned heading (meta_doc_section_missing clears and the "
    "heading text reappears on disk) while project_agent_doc_forbidden and "
    "root_capabilities_requires_product both survive the sync untouched and "
    "the sync command itself still exits 0 -- proving sync is a bounded, "
    "marker-scoped auto-repair that never silently relocates or deletes a "
    "matrix-placed file",
)

_AW_TOML = """[[projects]]
name = "tool"
path = "apps/tool"
td_path = "apps/tool/tech-design"
cap_path = "apps/tool/CAPABILITIES.md"
label = "app:tool"

[[projects.workspaces]]
name = "tool"
paths = ["apps/tool/**"]
target = "rust"
test_cmd = "cargo test -p tool"
"""

_FORBIDDEN_PROJECT_AGENTS_MD = (
    "# Forbidden project-local agent doc\n\n"
    "AGENTS.md must live only at the repository root, never at project layer.\n"
)

_FORBIDDEN_ROOT_CAPABILITIES_MD = (
    "# Root Capabilities (should be rejected)\n\n"
    "This repository root is not declared a product, so root CAPABILITIES.md "
    "is not an allowed root document.\n"
)

_HEADING_MARKER = "### Non-Core Features"
_BLOCK_END_MARKER = "<!-- aw:meta:project-capabilities:end -->"


def _git(root: Path, *args: str) -> subprocess.CompletedProcess[str]:
    completed = subprocess.run(
        ["git", *args], cwd=root, capture_output=True, text=True, check=False
    )
    if completed.returncode != 0:
        raise AssertionError(f"git {' '.join(args)} failed: {completed.stderr}")
    return completed


def _write_fixture(root: Path) -> None:
    (root / "aw.toml").write_text(_AW_TOML, encoding="utf-8")
    (root / "apps/tool").mkdir(parents=True)

    _git(root, "init")
    _git(root, "config", "user.email", "fixture@example.com")
    _git(root, "config", "user.name", "Fixture")
    _git(root, "add", "-A")
    _git(root, "commit", "-m", "fixture")


def verify() -> list[str]:
    with tempfile.TemporaryDirectory(prefix="aw-ec-meta-matrix-") as raw_root:
        root = Path(raw_root)
        _write_fixture(root)

        init_result = final_json(run_aw(root, "meta", "init", "--project", "tool"))
        assert init_result["status"] == "initialized", init_result

        capabilities_path = root / "apps/tool/CAPABILITIES.md"
        assert capabilities_path.exists(), "meta init did not scaffold project CAPABILITIES.md"

        baseline = final_json(
            run_aw(root, "meta", "check", "--project", "tool", expect_success=False)
        )
        baseline_codes = {finding["code"] for finding in baseline["findings"]}
        for forbidden_code in (
            "project_agent_doc_forbidden",
            "root_capabilities_requires_product",
            "meta_doc_section_missing",
        ):
            assert forbidden_code not in baseline_codes, baseline_codes

        # Introduce three independent matrix violations in one pass.
        (root / "apps/tool/AGENTS.md").write_text(_FORBIDDEN_PROJECT_AGENTS_MD, encoding="utf-8")
        (root / "CAPABILITIES.md").write_text(_FORBIDDEN_ROOT_CAPABILITIES_MD, encoding="utf-8")

        original = capabilities_path.read_text(encoding="utf-8")
        assert _HEADING_MARKER in original, original
        heading_at = original.index(_HEADING_MARKER)
        end_at = original.index(_BLOCK_END_MARKER, heading_at)
        stripped = original[:heading_at] + original[end_at:]
        assert _HEADING_MARKER not in stripped, stripped
        capabilities_path.write_text(stripped, encoding="utf-8")

        violated = final_json(
            run_aw(root, "meta", "check", "--project", "tool", expect_success=False)
        )
        findings_by_code = {finding["code"]: finding for finding in violated["findings"]}

        agent_doc = findings_by_code["project_agent_doc_forbidden"]
        assert agent_doc["path"] == "apps/tool/AGENTS.md", agent_doc
        assert agent_doc["axis"] == "placement", agent_doc
        assert agent_doc["severity"] == "blocker", agent_doc

        root_cap = findings_by_code["root_capabilities_requires_product"]
        assert root_cap["path"] == "CAPABILITIES.md", root_cap
        assert root_cap["axis"] == "placement", root_cap

        heading = findings_by_code["meta_doc_section_missing"]
        assert heading["path"] == "apps/tool/CAPABILITIES.md", heading
        assert _HEADING_MARKER in heading["message"], heading
        assert heading["axis"] == "schema", heading

        # Cluster 2: sync mechanically heals the matrix-owned heading but
        # leaves both placement violations for a human/agent to resolve.
        synced = run_aw(root, "meta", "sync", "--project", "tool")
        assert synced.returncode == 0, synced.stderr
        synced_json = final_json(synced)
        assert synced_json["status"] == "synchronized", synced_json
        synced_codes = {finding["code"] for finding in synced_json["findings"]}
        assert "meta_doc_section_missing" not in synced_codes, synced_codes
        assert "project_agent_doc_forbidden" in synced_codes, synced_codes
        assert "root_capabilities_requires_product" in synced_codes, synced_codes

        healed = capabilities_path.read_text(encoding="utf-8")
        assert _HEADING_MARKER in healed, healed
        assert (root / "apps/tool/AGENTS.md").exists(), "sync silently deleted the forbidden file"
        assert (root / "CAPABILITIES.md").exists(), "sync silently deleted the forbidden root file"

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
