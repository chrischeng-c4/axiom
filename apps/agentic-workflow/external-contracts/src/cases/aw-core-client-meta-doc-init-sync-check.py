"""Black-box contract for the META-doc init/sync/check producer control plane (#3306).

Drives real `aw meta init`/`check`/`sync` against a fixture project and
proves the one producer registry backing all three verbs: init scaffolds
every canonical repo- and project-layer skeleton and names an executable
`aw meta check` next command, check is genuinely read-only (byte-identical
file before and after a drift report), and sync repairs only the
marker-owned span -- preserving hand-authored bytes outside it and
converging to a byte-idempotent second run.
"""

from __future__ import annotations

import hashlib
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import final_json, run_aw

CASE_ID = "aw-core-client-meta-doc-init-sync-check"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "meta-doc-init-sync-check"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-meta-doc-init-sync-check"
)
ASSERTIONS = (
    "aw meta init on a fresh fixture scaffolds all four repo-layer skeletons "
    "(AGENTS.md, CLAUDE.md, README.md, CONTRIBUTING.md) and all three "
    "project-layer skeletons (README.md, CONTRIBUTING.md, CAPABILITIES.md) "
    "each marked created and names aw meta check as its next executable "
    "command, then hand-tampering text inside the project README's "
    "marker-owned span and running aw meta check reports a "
    "managed_block_stale finding naming the exact file, block id, and aw "
    "meta sync remediation while the file's on-disk bytes are identical "
    "before and after the check call -- proving check is genuinely "
    "read-only rather than merely advertised as such",
    "appending real hand-authored prose immediately after that same "
    "marker's owned end boundary and then running aw meta sync twice in a "
    "row shows the first sync repairs only the marker-owned span (the "
    "tampered canonical text is restored) while the trailing hand-authored "
    "prose survives byte-for-byte untouched, and the second sync produces "
    "byte-identical file content to the first and reports the block as "
    "unchanged -- proving sync is marker-scoped and byte-idempotent rather "
    "than a blanket rewrite of the whole document",
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

_END_MARKER = "<!-- aw:meta:project-readme:end -->"
_CANONICAL_LINE = "Product promises and work roots live in [CAPABILITIES.md](CAPABILITIES.md)."
_TAMPERED_LINE = "Product promises and work roots live in TAMPERED-AND-WRONG."
_HUMAN_TRAILER = (
    "\n## My Human Notes\n\n"
    "Hand-authored context that must survive an `aw meta sync` byte-for-byte.\n"
)

_EXPECTED_CREATED_SKELETONS = (
    ("CLAUDE.md", "repo-claude-guidance"),
    ("AGENTS.md", "repo-agents-guidance"),
    ("README.md", "repo-readme-skeleton"),
    ("CONTRIBUTING.md", "repo-contributing-skeleton"),
    ("apps/tool/README.md", "project-readme-skeleton"),
    ("apps/tool/CONTRIBUTING.md", "project-contributing-skeleton"),
    ("apps/tool/CAPABILITIES.md", "project-capabilities-skeleton"),
)


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


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify() -> list[str]:
    with tempfile.TemporaryDirectory(prefix="aw-ec-meta-producer-") as raw_root:
        root = Path(raw_root)
        _write_fixture(root)

        init_result = final_json(run_aw(root, "meta", "init", "--project", "tool"))
        assert init_result["status"] == "initialized", init_result
        assert init_result["next"]["command"] == "aw meta check --project tool", init_result

        created = {
            (change["path"], change["block"])
            for change in init_result["changes"]
            if change["status"] == "created"
        }
        for expected in _EXPECTED_CREATED_SKELETONS:
            assert expected in created, (expected, created)

        readme_path = root / "apps/tool/README.md"
        original = readme_path.read_text(encoding="utf-8")
        assert _CANONICAL_LINE in original, original
        assert _END_MARKER in original, original

        # Tamper inside the marker-owned span only.
        tampered = original.replace(_CANONICAL_LINE, _TAMPERED_LINE)
        assert tampered != original, original
        readme_path.write_text(tampered, encoding="utf-8")
        before_check_digest = _sha256(readme_path)

        checked = run_aw(root, "meta", "check", "--project", "tool", expect_success=False)
        checked_json = final_json(checked)
        findings_by_path = {finding["path"]: finding for finding in checked_json["findings"]}
        stale = findings_by_path["apps/tool/README.md"]
        assert stale["code"] == "managed_block_stale", stale
        assert "project-readme-skeleton" in stale["message"], stale
        assert "aw meta sync" in stale["remediation"], stale

        after_check_digest = _sha256(readme_path)
        assert after_check_digest == before_check_digest, "aw meta check wrote to disk"
        assert readme_path.read_text(encoding="utf-8") == tampered, "check mutated file content"

        # Append real hand-authored content immediately after the marker's
        # owned end boundary, preserving the in-span tamper.
        end_at = tampered.index(_END_MARKER) + len(_END_MARKER)
        with_human_trailer = tampered[:end_at] + _HUMAN_TRAILER + tampered[end_at:]
        readme_path.write_text(with_human_trailer, encoding="utf-8")

        first_sync = final_json(run_aw(root, "meta", "sync", "--project", "tool"))
        assert first_sync["status"] == "synchronized", first_sync
        first_sync_text = readme_path.read_text(encoding="utf-8")
        assert _CANONICAL_LINE in first_sync_text, first_sync_text
        assert _TAMPERED_LINE not in first_sync_text, first_sync_text
        assert _HUMAN_TRAILER.strip() in first_sync_text, first_sync_text
        first_sync_digest = _sha256(readme_path)

        second_sync = final_json(run_aw(root, "meta", "sync", "--project", "tool"))
        assert second_sync["status"] == "synchronized", second_sync
        second_sync_changes = {
            (change["path"], change["block"]): change["status"]
            for change in second_sync["changes"]
        }
        assert (
            second_sync_changes[("apps/tool/README.md", "project-readme-skeleton")] == "unchanged"
        ), second_sync_changes
        second_sync_digest = _sha256(readme_path)
        assert second_sync_digest == first_sync_digest, "second sync was not byte-idempotent"

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
