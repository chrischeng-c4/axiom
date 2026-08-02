"""Black-box contract for `aw review`'s evidence-driven project-profile resolution (#3310)."""

from __future__ import annotations

import json
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import run_aw

CASE_ID = "existing-project-standardization-project-profile-conformance-review"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "project-profile-conformance-review"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case existing-project-standardization-project-profile-conformance-review"
)
ASSERTIONS = (
    "aw review --project <p>, run against a project whose Cargo.toml declares "
    "no served-surface or state-owning dependency and carries no Dockerfile or "
    "k8s manifest, resolves kind_surface=cli with every other profile "
    "dimension not_applicable, proving profile resolution walks real evidence "
    "rather than defaulting to a service-shaped guess",
    "the identical aw review --project <p> command, run against a second "
    "project whose only difference is an axum dependency in Cargo.toml, "
    "resolves kind_surface=service / primary_workload=deployment / "
    "state_ownership=external_state / replication=none / serving_role=standard "
    "and cites the literal evidence entries 'served-surface dependencies: axum' "
    "and 'no owned-state dependency found', proving the two distinct outcomes "
    "come from the two projects' real dependency graphs -- never a hardcoded "
    "per-project-name lookup",
)


def _write_project(root: Path, name: str, cargo_dependencies_block: str) -> None:
    project_dir = root / name
    project_dir.mkdir(parents=True)
    (project_dir / "Cargo.toml").write_text(
        f'[package]\nname = "{name}"\nversion = "0.1.0"\nedition = "2021"\n\n'
        f"[dependencies]\n{cargo_dependencies_block}",
        encoding="utf-8",
    )
    (project_dir / "src").mkdir()
    (project_dir / "src" / "lib.rs").write_text("pub fn noop() {}\n", encoding="utf-8")


def verify() -> list[str]:
    with tempfile.TemporaryDirectory(prefix="aw-ec-review-profile-") as raw_root:
        root = Path(raw_root)
        (root / "aw.toml").write_text(
            "[[projects]]\n"
            'name = "bare"\n'
            'path = "bare"\n'
            'label = "app:bare"\n\n'
            "[[projects]]\n"
            'name = "web"\n'
            'path = "web"\n'
            'label = "app:web"\n',
            encoding="utf-8",
        )
        _write_project(root, "bare", "")
        _write_project(root, "web", 'axum = "0.7"\n')

        bare = run_aw(root, "review", "--project", "bare", "--pretty")
        bare_payload = json.loads(bare.stdout)
        assert bare_payload["outcome"] == "resolved", bare.stdout
        assert bare_payload["profile"]["kind_surface"] == "cli", bare.stdout
        assert bare_payload["profile"]["primary_workload"] == "not_applicable", bare.stdout
        assert bare_payload["profile"]["state_ownership"] == "not_applicable", bare.stdout
        assert bare_payload["profile"]["replication"] == "not_applicable", bare.stdout
        assert bare_payload["profile"]["serving_role"] == "not_applicable", bare.stdout

        web = run_aw(root, "review", "--project", "web", "--pretty")
        web_payload = json.loads(web.stdout)
        assert web_payload["outcome"] == "resolved", web.stdout
        assert web_payload["profile"]["kind_surface"] == "service", web.stdout
        assert web_payload["profile"]["primary_workload"] == "deployment", web.stdout
        assert web_payload["profile"]["state_ownership"] == "external_state", web.stdout
        assert web_payload["profile"]["replication"] == "none", web.stdout
        assert web_payload["profile"]["serving_role"] == "standard", web.stdout
        web_evidence_details = [entry["detail"] for entry in web_payload["evidence"]]
        assert "served-surface dependencies: axum" in web_evidence_details, web.stdout
        assert "no owned-state dependency found" in web_evidence_details, web.stdout

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
