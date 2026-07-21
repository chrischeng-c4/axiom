#!/usr/bin/env python3
# HANDWRITE-BEGIN gap="missing-generator:lumen-rig-process-resource-count" tracker="#2324" reason="Collect independent cross-platform FD, socket, and thread counts for non-vacuous endurance evidence."
"""Print one live process resource count for Rig's numeric stdout capture."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import subprocess


def proc_entries(pid: int, child: str) -> list[Path] | None:
    path = Path("/proc") / str(pid) / child
    if not path.is_dir():
        return None
    return list(path.iterdir())


def lsof_lines(pid: int) -> list[str]:
    result = subprocess.run(
        ["lsof", "-nP", "-p", str(pid)],
        check=True,
        capture_output=True,
        text=True,
        timeout=30,
    )
    lines = [line for line in result.stdout.splitlines() if line.strip()]
    if lines and lines[0].split()[:1] == ["COMMAND"]:
        lines = lines[1:]
    return lines


def file_descriptors(pid: int) -> int:
    entries = proc_entries(pid, "fd")
    if entries is not None:
        return len(entries)

    # lsof also reports pseudo entries such as cwd, txt, and mem. Only rows
    # whose FD column begins with a number represent an open file descriptor.
    return sum(
        1
        for line in lsof_lines(pid)
        if len(line.split()) >= 4 and line.split()[3][:1].isdigit()
    )


def sockets(pid: int) -> int:
    entries = proc_entries(pid, "fd")
    if entries is not None:
        count = 0
        for entry in entries:
            try:
                if os.readlink(entry).startswith("socket:"):
                    count += 1
            except OSError:
                continue
        return count

    socket_markers = {"IPv4", "IPv6", "unix"}
    return sum(
        1
        for line in lsof_lines(pid)
        if any(marker in line.split() for marker in socket_markers)
    )


def threads(pid: int) -> int:
    entries = proc_entries(pid, "task")
    if entries is not None:
        return len(entries)

    result = subprocess.run(
        ["ps", "-M", "-p", str(pid)],
        check=True,
        capture_output=True,
        text=True,
        timeout=30,
    )
    lines = [line for line in result.stdout.splitlines() if line.strip()]
    return max(len(lines) - 1, 0)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("metric", choices=("fd", "socket", "thread"))
    parser.add_argument("pid", type=int)
    args = parser.parse_args()

    if args.pid <= 0:
        raise SystemExit("pid must be positive")
    counters = {
        "fd": file_descriptors,
        "socket": sockets,
        "thread": threads,
    }
    print(counters[args.metric](args.pid))


if __name__ == "__main__":
    main()
# HANDWRITE-END
