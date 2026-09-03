#!/usr/bin/env python3
"""Run one acceptance session with a deadline and bounded shutdown grace."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import signal
import stat
import subprocess
import sys
import time


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--preflight-deadline-seconds", type=int, required=True)
    parser.add_argument("--deadline-seconds", type=int, required=True)
    parser.add_argument("--shutdown-grace-seconds", type=int, required=True)
    parser.add_argument("--ready-path", type=Path, required=True)
    parser.add_argument("--ready-token", required=True)
    parser.add_argument("--cleanup-ready-directory", type=Path)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    if (
        args.preflight_deadline_seconds <= 0
        or args.deadline_seconds <= 0
        or args.shutdown_grace_seconds <= 0
    ):
        parser.error("deadlines must be positive")
    if not args.ready_path.is_absolute():
        parser.error("ready path must be absolute")
    if args.cleanup_ready_directory is not None:
        if (
            not args.cleanup_ready_directory.is_absolute()
            or args.cleanup_ready_directory != args.ready_path.parent
        ):
            parser.error("cleanup ready directory must be the ready path parent")
    if not args.ready_token or "\n" in args.ready_token or "\t" in args.ready_token:
        parser.error("ready token must be one non-empty field")
    if args.command[:1] == ["--"]:
        args.command = args.command[1:]
    if not args.command:
        parser.error("a command is required")
    return args


def normalized_status(status: int) -> int:
    return 128 + (-status) if status < 0 else status


def ready_receipt_state(path: Path, token: str) -> str:
    """Return missing, ready, or invalid without following a symlink."""
    try:
        flags = os.O_RDONLY
        if hasattr(os, "O_NOFOLLOW"):
            flags |= os.O_NOFOLLOW
        descriptor = os.open(path, flags)
    except FileNotFoundError:
        return "missing"
    except OSError:
        return "invalid"
    try:
        metadata = os.fstat(descriptor)
        if not stat.S_ISREG(metadata.st_mode) or metadata.st_size > 4096:
            return "invalid"
        with os.fdopen(descriptor, "rb", closefd=False) as ready_file:
            body = ready_file.read(4097)
        expected = f"complete\t{token}\n".encode("ascii")
        return "ready" if body == expected else "invalid"
    except (OSError, UnicodeEncodeError):
        return "invalid"
    finally:
        os.close(descriptor)


def main() -> int:
    args = parse_args()
    child: subprocess.Popen[bytes] | None = None
    requested_signal: int | None = None
    timed_out = False
    protocol_failed = False

    def signal_group(signum: int) -> None:
        if child is None:
            return
        try:
            os.killpg(child.pid, signum)
        except ProcessLookupError:
            pass

    def forward(signum: int, _frame: object) -> None:
        nonlocal requested_signal
        if requested_signal is None:
            requested_signal = signum
        signal_group(signum)

    signal.signal(signal.SIGINT, forward)
    signal.signal(signal.SIGTERM, forward)
    if os.path.lexists(args.ready_path):
        print(
            "acceptance supervisor: ready receipt already exists",
            file=sys.stderr,
        )
        return 125
    child = subprocess.Popen(args.command, start_new_session=True)
    if requested_signal is not None:
        signal_group(requested_signal)

    preflight_deadline = time.monotonic() + args.preflight_deadline_seconds
    run_deadline: float | None = None
    shutdown_deadline: float | None = None
    forced_status: int | None = None
    while True:
        status = child.poll()
        if status is not None:
            break
        now = time.monotonic()
        if requested_signal is None and run_deadline is None:
            ready_state = ready_receipt_state(args.ready_path, args.ready_token)
            if ready_state == "ready":
                run_deadline = now + args.deadline_seconds
                print(
                    "acceptance supervisor: cloud-ready receipt accepted; "
                    "starting cloud deadline",
                    file=sys.stderr,
                    flush=True,
                )
            elif ready_state == "invalid":
                protocol_failed = True
                requested_signal = signal.SIGTERM
                signal_group(signal.SIGTERM)
                print(
                    "acceptance supervisor: invalid cloud-ready receipt; "
                    "starting bounded cleanup grace",
                    file=sys.stderr,
                    flush=True,
                )
            elif now >= preflight_deadline:
                timed_out = True
                requested_signal = signal.SIGTERM
                signal_group(signal.SIGTERM)
                print(
                    "acceptance supervisor: preflight deadline reached; "
                    "starting bounded cleanup grace",
                    file=sys.stderr,
                    flush=True,
                )
        if (
            requested_signal is None
            and run_deadline is not None
            and now >= run_deadline
        ):
            timed_out = True
            requested_signal = signal.SIGTERM
            signal_group(signal.SIGTERM)
            print(
                "acceptance supervisor: cloud deadline reached; "
                "starting bounded cleanup grace",
                file=sys.stderr,
                flush=True,
            )
        if requested_signal is not None and shutdown_deadline is None:
            shutdown_deadline = now + args.shutdown_grace_seconds
            if protocol_failed:
                forced_status = 125
            else:
                forced_status = 124 if timed_out else 128 + requested_signal
        if shutdown_deadline is not None and now >= shutdown_deadline:
            print(
                "acceptance supervisor: cleanup grace expired; "
                "killing the isolated process group",
                file=sys.stderr,
                flush=True,
            )
            signal_group(signal.SIGKILL)
            try:
                status = child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                status = None
            break
        time.sleep(0.05)

    # The session leader can exit before a descendant. Do not let such a
    # descendant survive either a normal run or a failed cleanup.
    signal_group(signal.SIGKILL)
    final_ready = run_deadline is not None or ready_receipt_state(
        args.ready_path, args.ready_token
    ) == "ready"
    try:
        args.ready_path.unlink(missing_ok=True)
        if args.cleanup_ready_directory is not None:
            args.cleanup_ready_directory.rmdir()
    except OSError:
        pass
    if forced_status is not None:
        return forced_status
    if requested_signal is not None:
        return 128 + requested_signal
    if status is None:
        return 124
    if normalized_status(status) == 0 and not final_ready:
        print(
            "acceptance supervisor: child exited successfully without "
            "a valid cloud-ready receipt",
            file=sys.stderr,
        )
        return 125
    return normalized_status(status)


if __name__ == "__main__":
    raise SystemExit(main())
