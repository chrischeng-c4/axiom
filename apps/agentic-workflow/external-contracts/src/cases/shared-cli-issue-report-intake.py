"""Python EC implementation for shared CLI Report intake labels."""

from __future__ import annotations

import json
import os
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse


CASE_ID = "shared-cli-issue-report-intake"
REPOSITORY_ROOT = Path(__file__).resolve().parents[5]
CAP_FULL = REPOSITORY_ROOT / "target" / "debug" / "cap-full"


class CourierCapture(BaseHTTPRequestHandler):
    request_path = ""
    request_body: dict[str, object] = {}

    def do_POST(self) -> None:  # noqa: N802 - BaseHTTPRequestHandler API
        length = int(self.headers["content-length"])
        type(self).request_path = self.path
        type(self).request_body = json.loads(self.rfile.read(length))
        response = json.dumps(
            {"html_url": "https://github.com/chrischeng-c4/axiom/issues/999999"}
        ).encode()
        self.send_response(201)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(response)))
        self.end_headers()
        self.wfile.write(response)

    def log_message(self, _format: str, *_args: object) -> None:
        pass


def run_cap(*args: str, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    completed = subprocess.run(
        [str(CAP_FULL), "issue", "create", *args],
        cwd=REPOSITORY_ROOT,
        env=env,
        input="y\n",
        text=True,
        capture_output=True,
        check=False,
    )
    assert completed.returncode == 0, completed.stdout + completed.stderr
    return completed


def labels_from_preview(stdout: str) -> set[str]:
    labels_line = next(line for line in stdout.splitlines() if line.startswith("labels:"))
    return {
        label.strip() for label in labels_line.removeprefix("labels:").split(",")
    }


def verify() -> list[str]:
    build = subprocess.run(
        ["cargo", "build", "-p", "cap", "--bin", "cap-full"],
        cwd=REPOSITORY_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    assert build.returncode == 0, build.stdout + build.stderr

    preview = run_cap(
        "--title",
        "Shared CLI intake",
        "--dry-run",
        "Observed behavior",
    )
    assert labels_from_preview(preview.stdout) == {"app:cap", "type:report"}
    assert preview.stdout.rstrip().endswith("next: done")

    CourierCapture.request_path = ""
    CourierCapture.request_body = {}
    server = ThreadingHTTPServer(("127.0.0.1", 0), CourierCapture)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        courier_env = os.environ.copy()
        courier_env["AXIOM_COURIER_URL"] = (
            f"http://127.0.0.1:{server.server_address[1]}"
        )
        courier = run_cap(
            "--title",
            "Shared CLI courier intake",
            "Observed courier behavior",
            env=courier_env,
        )
    finally:
        server.shutdown()
        thread.join()
        server.server_close()
    assert CourierCapture.request_path == "/v1/issues/chrischeng-c4/axiom"
    assert set(CourierCapture.request_body["labels"]) == {
        "app:cap",
        "type:report",
    }
    assert courier.stdout.rstrip().endswith("next: done")

    fallback_env = os.environ.copy()
    for name in ("AXIOM_COURIER_URL", "AXIOM_COURIER_TOKEN", "GH_TOKEN", "GITHUB_TOKEN"):
        fallback_env.pop(name, None)
    fallback_env["PATH"] = "/usr/bin:/bin"
    fallback = run_cap(
        "--title",
        "Shared CLI fallback intake",
        "Observed fallback behavior",
        env=fallback_env,
    )
    fallback_url = next(
        line for line in fallback.stdout.splitlines() if line.startswith("https://")
    )
    fallback_labels = set(parse_qs(urlparse(fallback_url).query)["labels"][0].split(","))
    assert fallback_labels == {"app:cap", "type:report"}
    assert fallback.stdout.rstrip().endswith("next: done")

    return [
        "the real shared CLI dry-run exposes app identity and type:report",
        "the courier transport sends both labels in its JSON issue payload",
        "the credential-free fallback preserves both labels in its issue URL",
        "all exercised public paths preserve the executable terminal next marker",
    ]
