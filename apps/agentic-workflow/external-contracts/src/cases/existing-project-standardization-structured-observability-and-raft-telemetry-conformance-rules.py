"""Black-box contract for `aw review`'s obs baseline + raft telemetry rules (#3310)."""

from __future__ import annotations

import json
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import run_aw

CASE_ID = (
    "existing-project-standardization-structured-observability-and-raft-telemetry-"
    "conformance-rules"
)
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "structured-observability-and-raft-telemetry-conformance-rules"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case existing-project-standardization-structured-observability-and-raft-telemetry-"
    "conformance-rules"
)
ASSERTIONS = (
    "aw review --project <p> --pretty, run against a served-surface project "
    "with no service-observability Cargo dependency, reports "
    "'obs:structured-logging-metrics-adoption' as a mandatory-baseline finding "
    "even though the source tree has no hand-rolled logging code at all, and "
    "declaring a service-observability dependency (source unchanged) removes "
    "that exact finding on the next run, proving the observability baseline "
    "rule is a real dependency-graph read rather than an anti-pattern-only "
    "heuristic or a permanently-tripped flag",
    "aw review --project <p> --pretty, run against a StatefulSet/RaftConsensus "
    "profiled project (stateful_storage trait, a raft-runtime dependency, and a "
    "leader_ingest source marker) whose source carries none of the proposal-"
    "routing telemetry markers, reports "
    "'raft:proposal-routing-telemetry-gap', and adding a single "
    "local_proposals counter marker to that same source file removes the "
    "finding on the next run, proving raft telemetry conformance is scoped to "
    "profiles that are actually RaftConsensus/leader-ingest shaped and is "
    "evaluated from real source content",
)

_INGEST_RS_NO_TELEMETRY = (
    "// All publishes are forwarded to the leader (leader_ingest): only the\n"
    "// leader accepts writes.\n"
    "pub fn leader_ingest() {}\n"
)
_INGEST_RS_WITH_TELEMETRY = (
    "// All publishes are forwarded to the leader (leader_ingest): only the\n"
    "// leader accepts writes.\n"
    "pub fn leader_ingest() {\n"
    "    let _ = local_proposals();\n"
    "}\n"
    "fn local_proposals() -> u64 { 0 }\n"
)


def _write_web_project(root: Path, cargo_dependencies_block: str) -> None:
    project_dir = root / "web"
    project_dir.mkdir(parents=True, exist_ok=True)
    (project_dir / "Cargo.toml").write_text(
        '[package]\nname = "web"\nversion = "0.1.0"\nedition = "2021"\n\n'
        f"[dependencies]\naxum = \"0.7\"\n{cargo_dependencies_block}",
        encoding="utf-8",
    )
    (project_dir / "src").mkdir(exist_ok=True)
    (project_dir / "src" / "lib.rs").write_text("pub fn noop() {}\n", encoding="utf-8")


def _write_raftnode_project(root: Path, ingest_rs: str) -> None:
    project_dir = root / "raftnode"
    project_dir.mkdir(parents=True, exist_ok=True)
    (project_dir / "aw.toml").write_text(
        '[project]\nname = "fixture"\n\n[capability.profile]\n'
        'traits = ["service", "network_exposed", "stateful_storage"]\n',
        encoding="utf-8",
    )
    (project_dir / "Cargo.toml").write_text(
        '[package]\nname = "raftnode"\nversion = "0.1.0"\nedition = "2021"\n\n'
        '[dependencies]\nraft-runtime = "1"\n',
        encoding="utf-8",
    )
    (project_dir / "src").mkdir(exist_ok=True)
    (project_dir / "src" / "ingest.rs").write_text(ingest_rs, encoding="utf-8")


def verify() -> list[str]:
    with tempfile.TemporaryDirectory(prefix="aw-ec-review-obs-raft-") as raw_root:
        root = Path(raw_root)
        (root / "aw.toml").write_text(
            '[[projects]]\nname = "web"\npath = "web"\nlabel = "app:web"\n\n'
            '[[projects]]\nname = "raftnode"\npath = "raftnode"\nlabel = "app:raftnode"\n',
            encoding="utf-8",
        )

        _write_web_project(root, "")
        obs_before = run_aw(root, "review", "--project", "web", "--pretty")
        obs_before_payload = json.loads(obs_before.stdout)
        assert any(
            f["id"] == "obs:structured-logging-metrics-adoption"
            for f in obs_before_payload["findings"]
        ), obs_before.stdout

        _write_web_project(root, 'service-observability = { path = "../service-observability" }\n')
        obs_after = run_aw(root, "review", "--project", "web", "--pretty")
        obs_after_payload = json.loads(obs_after.stdout)
        assert all(
            f["id"] != "obs:structured-logging-metrics-adoption"
            for f in obs_after_payload["findings"]
        ), obs_after.stdout

        _write_raftnode_project(root, _INGEST_RS_NO_TELEMETRY)
        raft_before = run_aw(root, "review", "--project", "raftnode", "--pretty")
        raft_before_payload = json.loads(raft_before.stdout)
        assert raft_before_payload["profile"]["replication"] == "raft_consensus", (
            raft_before.stdout
        )
        assert any(
            f["id"] == "raft:proposal-routing-telemetry-gap"
            for f in raft_before_payload["findings"]
        ), raft_before.stdout

        _write_raftnode_project(root, _INGEST_RS_WITH_TELEMETRY)
        raft_after = run_aw(root, "review", "--project", "raftnode", "--pretty")
        raft_after_payload = json.loads(raft_after.stdout)
        assert all(
            f["id"] != "raft:proposal-routing-telemetry-gap"
            for f in raft_after_payload["findings"]
        ), raft_after.stdout

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
