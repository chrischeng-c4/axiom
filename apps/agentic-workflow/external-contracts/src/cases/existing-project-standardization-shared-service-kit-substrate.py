"""Black-box contract for the shared HTTP service-kit substrate (#3310).

Drives `libs/service-http/examples/minimal_service` -- the owned real subprocess
wiring only production `service-http` / `server-lifecycle` / `server-tcp` /
`transport-h2c` types, mirroring `service-http`'s own "what a service wires"
doc example -- and proves the composed substrate is a genuinely running HTTP
policy shell with real graceful-shutdown semantics, not a compile-only or
doc-only abstraction.
"""

from __future__ import annotations

import queue
import re
import signal
import subprocess
import sys
import threading
import time
import urllib.error
import urllib.request
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import REPOSITORY_ROOT

CASE_ID = "existing-project-standardization-shared-service-kit-substrate"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "shared-service-kit-substrate"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case existing-project-standardization-shared-service-kit-substrate"
)
ASSERTIONS = (
    "libs/service-http/examples/minimal_service -- the same production "
    "service-http/server-lifecycle/server-tcp/transport-h2c composition "
    "lumen, keep, relay, and loom each wire (service-http's own 'what a "
    "service wires' doc example) -- run as a real subprocess binds a "
    "loopback port and serves /healthz, /readyz, /metrics, /openapi.json, "
    "and /docs with 200s plus a Server-Timing response header, proving the "
    "shared substrate is a genuine running HTTP policy shell rather than a "
    "compile-only or doc-only abstraction",
    "sending that subprocess a real SIGTERM flips /readyz to 503 "
    "'draining' immediately while the listener stays open, then the "
    "process holds its configured grace window before it actually exits "
    "and prints its final shutdown line, proving server-lifecycle's drain "
    "signal and grace-window shutdown genuinely gate the shared runtime's "
    "termination instead of the process exiting immediately or /readyz "
    "silently staying healthy while connections are cut",
)

_LISTENING_RE = re.compile(r"^LISTENING (\S+)$")
_STARTUP_TIMEOUT_S = 15.0
_EXIT_TIMEOUT_S = 15.0
_GRACE_SECS = 2


def _drain_stdout(pipe, sink: "queue.Queue[str]") -> None:
    for line in iter(pipe.readline, ""):
        sink.put(line)
    pipe.close()


def _read_body(url: str) -> tuple[int, str, dict[str, str]]:
    try:
        with urllib.request.urlopen(url, timeout=5) as resp:
            return resp.status, resp.read().decode("utf-8", "replace"), dict(resp.headers)
    except urllib.error.HTTPError as exc:
        return exc.code, exc.read().decode("utf-8", "replace"), dict(exc.headers or {})


def verify() -> list[str]:
    build = subprocess.run(
        ["cargo", "build", "-p", "service-http", "--example", "minimal_service"],
        cwd=REPOSITORY_ROOT,
        capture_output=True,
        text=True,
        timeout=1800,
    )
    assert build.returncode == 0, f"example build failed:\n{build.stdout}\n{build.stderr}"

    binary = REPOSITORY_ROOT / "target" / "debug" / "examples" / "minimal_service"
    assert binary.exists(), f"missing built example binary at {binary}"

    proc = subprocess.Popen(
        [str(binary)],
        cwd=REPOSITORY_ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
    )
    lines: "queue.Queue[str]" = queue.Queue()
    reader = threading.Thread(target=_drain_stdout, args=(proc.stdout, lines), daemon=True)
    reader.start()

    try:
        addr = None
        captured: list[str] = []
        deadline = time.monotonic() + _STARTUP_TIMEOUT_S
        while time.monotonic() < deadline and addr is None:
            try:
                line = lines.get(timeout=0.2)
            except queue.Empty:
                continue
            captured.append(line)
            match = _LISTENING_RE.match(line.strip())
            if match:
                addr = match.group(1)
        assert addr is not None, f"never observed a LISTENING line; captured: {captured}"

        base = f"http://{addr}"

        status, body, _headers = _read_body(f"{base}/healthz")
        assert status == 200, (status, body)

        status, body, _headers = _read_body(f"{base}/readyz")
        assert status == 200, (status, body)

        status, body, _headers = _read_body(f"{base}/metrics")
        assert status == 200, (status, body)

        status, body, headers = _read_body(f"{base}/healthz")
        assert status == 200, (status, body)
        assert "server-timing" in {k.lower() for k in headers}, headers

        status, body, _headers = _read_body(f"{base}/openapi.json")
        assert status == 200, (status, body)
        assert '"openapi"' in body, body

        status, body, _headers = _read_body(f"{base}/docs")
        assert status == 200, (status, body)
        assert "swagger-ui" in body, body
        assert "/openapi.json" in body, body

        signal_at = time.monotonic()
        proc.send_signal(signal.SIGTERM)

        draining_status = None
        draining_deadline = time.monotonic() + 2.0
        while time.monotonic() < draining_deadline:
            status, body, _headers = _read_body(f"{base}/readyz")
            if status == 503:
                draining_status = (status, body)
                break
            time.sleep(0.05)
        assert draining_status is not None, "readyz never flipped to 503 after SIGTERM"
        assert "draining" in draining_status[1], draining_status

        try:
            returncode = proc.wait(timeout=_EXIT_TIMEOUT_S)
        except subprocess.TimeoutExpired:
            raise AssertionError(
                f"process did not exit within {_EXIT_TIMEOUT_S}s of SIGTERM "
                f"despite only a {_GRACE_SECS}s configured grace window"
            )
        elapsed = time.monotonic() - signal_at
        assert returncode == 0, f"example exited with {returncode}"
        assert elapsed >= (_GRACE_SECS - 0.5), (
            f"process exited after only {elapsed:.2f}s, before its "
            f"{_GRACE_SECS}s grace window could plausibly have elapsed"
        )

        drain_deadline = time.monotonic() + 2.0
        while time.monotonic() < drain_deadline:
            try:
                captured.append(lines.get(timeout=0.2))
            except queue.Empty:
                continue
        assert any("SHUTDOWN complete" in line for line in captured), captured
    finally:
        if proc.poll() is None:
            proc.kill()
            proc.wait(timeout=5)

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
