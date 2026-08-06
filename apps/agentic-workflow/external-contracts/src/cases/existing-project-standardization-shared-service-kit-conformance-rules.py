"""Black-box contract for `aw review`'s R1 shared-service-kit adoption rules (#3310)."""

from __future__ import annotations

import json
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import run_aw

CASE_ID = "existing-project-standardization-shared-service-kit-conformance-rules"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "shared-service-kit-conformance-rules"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case existing-project-standardization-shared-service-kit-conformance-rules"
)
ASSERTIONS = (
    "aw review --project <p>, run against a served-surface project whose "
    "source tree contains a hand-rolled 'TcpListener::bind' marker and whose "
    "Cargo.toml declares none of libs/server-tcp, server-http, transport-h2c, "
    "or service-http, reports a 'shared-kit:server-tcp' finding at "
    "high severity naming the offending source path, proving the rule fires "
    "from a real marker-present-and-dependency-absent evaluation",
    "adding a server-tcp dependency to that same project's Cargo.toml, with "
    "the offending source file byte-for-byte unchanged, removes the "
    "'shared-kit:server-tcp' finding from the next aw review --project <p> "
    "run, proving shared-service-kit adoption is checked by a live "
    "dependency-graph read rather than a permanently-tripped source-marker "
    "flag",
)

_SERVER_RS = (
    "pub fn serve() {\n"
    '    let _listener = std::net::TcpListener::bind("0.0.0.0:8080").unwrap();\n'
    "}\n"
)


def _write_project(root: Path, cargo_dependencies_block: str) -> None:
    project_dir = root / "web"
    project_dir.mkdir(parents=True, exist_ok=True)
    (project_dir / "Cargo.toml").write_text(
        '[package]\nname = "web"\nversion = "0.1.0"\nedition = "2021"\n\n'
        f"[dependencies]\naxum = \"0.7\"\n{cargo_dependencies_block}",
        encoding="utf-8",
    )
    (project_dir / "src").mkdir(exist_ok=True)
    (project_dir / "src" / "server.rs").write_text(_SERVER_RS, encoding="utf-8")


def verify() -> list[str]:
    with tempfile.TemporaryDirectory(prefix="aw-ec-review-kit-rules-") as raw_root:
        root = Path(raw_root)
        (root / "aw.toml").write_text(
            '[[projects]]\nname = "web"\npath = "web"\nlabel = "app:web"\n',
            encoding="utf-8",
        )
        _write_project(root, "")

        before = run_aw(root, "review", "--project", "web", "--pretty")
        before_payload = json.loads(before.stdout)
        server_tcp_findings = [
            f for f in before_payload["findings"] if f["id"] == "shared-kit:server-tcp"
        ]
        assert len(server_tcp_findings) == 1, before.stdout
        assert server_tcp_findings[0]["severity"] == "high", before.stdout
        assert "src/server.rs" in server_tcp_findings[0]["affected_paths"], before.stdout

        _write_project(root, 'server-tcp = { path = "../server-tcp" }\n')

        after = run_aw(root, "review", "--project", "web", "--pretty")
        after_payload = json.loads(after.stdout)
        assert all(f["id"] != "shared-kit:server-tcp" for f in after_payload["findings"]), (
            after.stdout
        )

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
