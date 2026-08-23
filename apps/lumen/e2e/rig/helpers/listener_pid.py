#!/usr/bin/env python3
# HANDWRITE-BEGIN gap="missing-generator:lumen-rig-listener-pid" tracker="#2324" reason="Resolve the exact VAT-managed listener process so soak evidence cannot sample an unrelated same-name process."
"""Resolve the unique process listening on a local TCP endpoint."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import subprocess


def linux_listener_pids(port: int) -> set[int] | None:
    tables = [Path("/proc/net/tcp"), Path("/proc/net/tcp6")]
    if not any(table.is_file() for table in tables):
        return None

    socket_inodes: set[str] = set()
    expected_port = f"{port:04X}"
    for table in tables:
        if not table.is_file():
            continue
        for line in table.read_text().splitlines()[1:]:
            columns = line.split()
            if len(columns) < 10:
                continue
            local_address, state, inode = columns[1], columns[3], columns[9]
            if local_address.rsplit(":", 1)[-1] == expected_port and state == "0A":
                socket_inodes.add(inode)

    pids: set[int] = set()
    for process in Path("/proc").iterdir():
        if not process.name.isdigit():
            continue
        fd_dir = process / "fd"
        try:
            descriptors = list(fd_dir.iterdir())
        except OSError:
            continue
        for descriptor in descriptors:
            try:
                target = os.readlink(descriptor)
            except OSError:
                continue
            if target.startswith("socket:[") and target[8:-1] in socket_inodes:
                pids.add(int(process.name))
                break
    return pids


def lsof_listener_pids(port: int) -> set[int]:
    result = subprocess.run(
        ["lsof", "-nP", f"-iTCP:{port}", "-sTCP:LISTEN", "-t"],
        check=True,
        capture_output=True,
        text=True,
        timeout=30,
    )
    return {int(line) for line in result.stdout.splitlines() if line.strip()}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("endpoint", help="local host:port, for example 127.0.0.1:7373")
    args = parser.parse_args()
    try:
        port = int(args.endpoint.rsplit(":", 1)[1])
    except (IndexError, ValueError) as error:
        raise SystemExit(f"invalid endpoint: {args.endpoint}") from error

    pids = linux_listener_pids(port)
    if pids is None:
        pids = lsof_listener_pids(port)
    if len(pids) != 1:
        raise SystemExit(
            f"expected exactly one listener on {args.endpoint}, found {sorted(pids)}"
        )
    print(next(iter(pids)))


if __name__ == "__main__":
    main()
# HANDWRITE-END
