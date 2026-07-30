"""Process-level harness for Tape external contracts.

Everything here talks to a Tape server the way an external client does: over
HTTP, against a binary started as a child process. Nothing imports Tape's Rust
source, and nothing reads Tape's on-disk files. That restriction is the point:
a verifier that inspects the server's own bookkeeping to decide whether the
server's bookkeeping is correct proves nothing.

What the kill in `Server.kill9` does and does not establish is worth being
exact about, because it is easy to overclaim. SIGKILL destroys the *process*.
It does not touch the page cache, which belongs to the kernel, so bytes handed
to `write(2)` and never fsynced are still there for the next process to read --
measured on this host: 3200 bytes written by a child, never synced, never
closed, fully readable after the child was SIGKILLed. So these contracts prove
crash consistency across process death; they do not, and cannot, prove what
survives a power loss. A build that acknowledges before its durability barrier
would pass every recovery assertion here. The thing that catches *that* is the
barrier ceiling in `ec-3052-durable-append-scaling.py`, which fails a build
whose throughput is arithmetically impossible for one that actually waits.

Standard library only, deliberately: a durability oracle that needs a
dependency resolver to run is one more thing that can be broken or skipped on
the machine where the evidence actually matters.
"""

from __future__ import annotations

import json
import os
import signal
import socket
import subprocess
import time
import urllib.error
import urllib.request
from dataclasses import dataclass
from pathlib import Path

# src/ -> external-contracts/ -> tape/ -> apps/ -> repo root
REPO_ROOT = Path(__file__).resolve().parents[4]
TAPE_DIR = REPO_ROOT / "apps" / "tape"
EVIDENCE_DIR = TAPE_DIR / "external-contracts" / "evidence"


class ContractFailure(AssertionError):
    """A contract assertion failed. Distinct from a harness bug on purpose.

    Carries the measurements that produced the verdict, not just the sentence
    describing it. A red efficiency contract whose evidence records only prose
    leaves no before-number to compare the after-number against, which is most
    of what a pre-change red run is for.
    """

    def __init__(self, message: str, facts: dict | None = None) -> None:
        super().__init__(message)
        self.facts = facts or {}


def build_binary(release: bool = False) -> Path:
    """Build and return the `tape` binary path.

    The durability contract uses debug: it asserts what survives a kill, which
    optimisation does not change, and debug builds faster.

    The scaling contract must use release. It measures a *ratio* between
    concurrency levels, and in a debug build the server's own per-request cost
    is inflated enough to become the limiter once the fsync barrier is
    amortised away -- which would hide exactly the improvement the contract is
    there to observe. Neither case reports an absolute throughput number:
    #3052 forbids capacity claims from any measurement not taken on target
    hardware, and this harness runs on a developer laptop.
    """
    profile = "release" if release else "debug"
    command = ["cargo", "build", "-q", "-p", "tape", "--bin", "tape"]
    if release:
        command.append("--release")
    subprocess.run(command, cwd=REPO_ROOT, check=True)
    binary = REPO_ROOT / "target" / profile / "tape"
    if not binary.is_file():
        raise ContractFailure(f"cargo reported success but {binary} is missing")
    return binary


def free_port() -> int:
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


@dataclass
class Server:
    """A running `tape serve` child process."""

    process: subprocess.Popen
    port: int
    data_dir: Path
    log_path: Path

    @property
    def base(self) -> str:
        return f"http://127.0.0.1:{self.port}"

    def kill9(self) -> None:
        """SIGKILL. No drain, no signal handler, no chance to flush.

        This is the whole reason these contracts run out-of-process. A Rust
        integration test cannot SIGKILL itself and then observe what it left
        behind, so the strongest durability claim tape's in-process suite can
        make is that its own shutdown path is orderly. That is not the claim
        anyone cares about.
        """
        self.process.send_signal(signal.SIGKILL)
        self.process.wait(timeout=30)

    def terminate(self) -> None:
        if self.process.poll() is None:
            self.process.send_signal(signal.SIGKILL)
            self.process.wait(timeout=30)

    def tail(self, lines: int = 30) -> str:
        try:
            return "\n".join(self.log_path.read_text().splitlines()[-lines:])
        except OSError:
            return "<no log>"


def start_server(
    binary: Path,
    data_dir: Path,
    log_path: Path,
    port: int | None = None,
    extra_env: dict[str, str] | None = None,
) -> Server:
    """Start `tape serve --data-dir` and wait until it answers /readyz."""
    port = port or free_port()
    env = dict(os.environ)
    env["RUST_LOG"] = env.get("RUST_LOG", "warn")
    if extra_env:
        env.update(extra_env)
    handle = log_path.open("ab")
    process = subprocess.Popen(
        [
            str(binary),
            "serve",
            "--bind",
            f"127.0.0.1:{port}",
            "--data-dir",
            str(data_dir),
            "--auth",
            "off",
        ],
        cwd=REPO_ROOT,
        env=env,
        stdout=handle,
        stderr=handle,
    )
    server = Server(process=process, port=port, data_dir=data_dir, log_path=log_path)
    deadline = time.monotonic() + 60
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise ContractFailure(
                f"tape serve exited with {process.returncode} before becoming "
                f"ready:\n{server.tail()}"
            )
        try:
            with urllib.request.urlopen(f"{server.base}/readyz", timeout=2) as response:
                if response.status == 200:
                    return server
        except (urllib.error.URLError, OSError):
            pass
        time.sleep(0.1)
    server.terminate()
    raise ContractFailure(f"tape serve never became ready:\n{server.tail()}")


def append(base: str, topic: str, key: str, payload: dict, timeout: float = 30.0) -> int:
    """POST one event. Returns the HTTP status the server answered with.

    The `HTTPError` catch is load-bearing, not defensive tidying.
    `urllib.request.urlopen` raises on every non-2xx, so without it this
    function can only ever *return* a 2xx and every refusal reaches the caller
    as an exception indistinguishable from a dead socket. The durability
    contract partitions its keys on exactly that distinction -- a 507 from
    `enforce_storage_writable` is a promise the server will NOT keep the write,
    while a lost connection is genuinely unknown -- so collapsing the two made
    its `refused` set permanently empty and two of its assertions vacuous.

    Returning `error.code` is safe on the post-kill path that motivated the
    original catch-all: a dead server's port resets the connection rather than
    answering, which surfaces as `URLError`/`OSError` and still propagates.
    """
    return append_raw(
        base, topic, json.dumps({"key": key, "payload": payload}).encode(), timeout
    )


def append_raw(base: str, topic: str, body: bytes, timeout: float = 30.0) -> int:
    """POST an arbitrary body to the append route. Returns the status.

    Exists so a contract can drive the server's *refusal* path on demand. A
    truncated body is refused with 400 by `serde_json::from_slice` before the
    handler reaches the journal, which is a deterministic non-2xx from a
    perfectly healthy server -- no fault injection, no size-capped filesystem,
    no host dependency. The durability contract uses it to keep its
    "a refused write must not come back" assertion live in every run instead of
    latent; see that file's docstring.
    """
    request = urllib.request.Request(
        f"{base}/topics/{topic}/append",
        data=body,
        headers={"content-type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            response.read()
            return int(response.status)
    except urllib.error.HTTPError as error:
        error.read()
        error.close()
        return int(error.code)


def replay_all(base: str, topic: str, page: int = 1000) -> list[dict]:
    """Page through the whole topic. `limit` defaults to 1000, so paging is required."""
    events: list[dict] = []
    offset = 0
    while True:
        url = f"{base}/topics/{topic}/replay?from_offset={offset}&limit={page}"
        with urllib.request.urlopen(url, timeout=60) as response:
            batch = json.loads(response.read())["events"]
        if not batch:
            return events
        events.extend(batch)
        offset = int(batch[-1]["offset"]) + 1


def write_evidence(case_id: str, verdict: str, facts: dict) -> Path:
    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)
    path = EVIDENCE_DIR / f"{case_id}.json"
    path.write_text(
        json.dumps(
            {"case_id": case_id, "verdict": verdict, "facts": facts},
            indent=2,
            sort_keys=True,
        )
        + "\n"
    )
    return path
