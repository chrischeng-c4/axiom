"""Black-box contract for the Python artifact-model compatibility parser (#3298)."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import final_json, project_fixture, run_aw


CASE_ID = "workflow-root-runner-python-artifact-model-compatibility-parser"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "python-artifact-model-compatibility-parser"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case workflow-root-runner-python-artifact-model-compatibility-parser"
)
ASSERTIONS = (
    "a project row that omits spec_model entirely, one that sets the historical spec_model = \"legacy\", and one that sets the historical spec_model = \"python-v1\" spelling are each independently accepted by the real aw health surface and produce byte-identical 'Python Spec project ... is authoritative' advisory notes in its full untruncated report, proving none of the three read-compatible spellings ever disables the canonical Python lifecycle",
    "a root project row that is itself stale (spec_model = \"legacy\") plus an independently discovered project-local overlay aw.toml that omits spec_model entirely still merges cleanly through the real two-file project registry and produces the identical advisory notes, proving a local overlay has no authority to disable or re-route the canonical lifecycle either",
    "independently, a project row whose spec_model is an unrecognized value makes the same real aw health surface fail closed with a distinct, real configuration-parse rejection naming the offending aw.toml file, unlike any of the accepted spellings above",
)

_ROOT_TOML_TEMPLATE = """[agentic_workflow.workspace]
mode = "in_place"

[agentic_workflow.issue_platform]
type = "local"

[[projects]]
name = "demo"
label = "app:demo"
path = "{path}"
tech_design_path = "tech-design"
{spec_model_line}
[[projects.workspaces]]
name = "demo"
paths = ["**"]
target = "rust"
"""

_OVERLAY_TOML = """[project]
name = "demo"
"""


def _write_root(root: Path, *, spec_model: str | None, path: str = ".") -> None:
    spec_model_line = f'spec_model = "{spec_model}"\n' if spec_model is not None else ""
    (root / "aw.toml").write_text(
        _ROOT_TOML_TEMPLATE.format(path=path, spec_model_line=spec_model_line),
        encoding="utf-8",
    )


def _health_envelope(root: Path) -> tuple[dict[str, object], int]:
    completed = run_aw(root, "health", "--project", "demo", "--json", expect_success=None)
    return final_json(completed), completed.returncode


def _health_notes(root: Path) -> dict[str, object]:
    # A brand-new fixture project is never itself production-healthy (it has
    # no capability document, no EC inventory, ...), so the command legitimately
    # exits non-zero -- what matters here is only whether the *configuration*
    # parsed and which advisory notes the full, untruncated health report
    # (named by the compact envelope's own `payload_path`) carries.
    envelope, _returncode = _health_envelope(root)
    assert "payload_path" in envelope, envelope
    full_report = json.loads(Path(str(envelope["payload_path"])).read_text(encoding="utf-8"))
    return {
        "traceability_note": full_report.get("traceability_note"),
        "cb_verify_note": full_report.get("cb_verify_note"),
    }


def verify() -> list[str]:
    expected_traceability_note = (
        "legacy source-marker traceability is advisory for Python Spec project `demo`; "
        "EC/TD semantic health is authoritative"
    )
    expected_cb_verify_note = (
        "legacy Markdown CB replay is advisory for Python Spec project `demo`; "
        "Python artifact readiness is authoritative"
    )

    # Cluster 1: an omitted spec_model, the historical `legacy` spelling, and
    # the historical `python-v1` spelling are each independently readable and
    # produce byte-identical advisory notes -- none of the three ever routes
    # the project back to the retired Markdown lifecycle.
    for spec_model in (None, "legacy", "python-v1"):
        with project_fixture() as root:
            _write_root(root, spec_model=spec_model)
            notes = _health_notes(root)
            assert notes["traceability_note"] == expected_traceability_note, (spec_model, notes)
            assert notes["cb_verify_note"] == expected_cb_verify_note, (spec_model, notes)

    # Cluster 2: a stale root value (`legacy`) plus an independently
    # discovered project-local overlay `aw.toml` (real two-file project
    # registry discovery: `apps/<name>/aw.toml`, mirroring this repo's own
    # `apps/agentic-workflow/aw.toml`) that itself omits spec_model still
    # merges cleanly and produces the identical advisory notes -- a local
    # overlay has no authority to disable or re-route the canonical
    # lifecycle either, whether or not it expresses its own opinion.
    with project_fixture() as root:
        _write_root(root, spec_model="legacy", path="apps/demo")
        overlay_dir = root / "apps" / "demo"
        overlay_dir.mkdir(parents=True, exist_ok=True)
        assert "spec_model" not in _OVERLAY_TOML
        (overlay_dir / "aw.toml").write_text(_OVERLAY_TOML, encoding="utf-8")
        notes = _health_notes(root)
        assert notes["traceability_note"] == expected_traceability_note, notes
        assert notes["cb_verify_note"] == expected_cb_verify_note, notes

    # Cluster 3: independently, an unrecognized spec_model value is rejected
    # outright -- a real configuration-parse failure, unlike any of the
    # accepted spellings exercised above.
    with project_fixture() as root:
        _write_root(root, spec_model="bogus-unknown-model")
        envelope, returncode = _health_envelope(root)
        assert returncode != 0, envelope
        assert envelope.get("status") == "blocked", envelope
        assert "payload_path" not in envelope, envelope
        reason = str(envelope["next"]["reason"])
        assert "could not be resolved: parsing" in reason, envelope
        assert reason.endswith("aw.toml"), envelope

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
