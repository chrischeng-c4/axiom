#!/usr/bin/env python3
"""Copy acceptance output to a durable log and write a success receipt."""

from __future__ import annotations

import os
from pathlib import Path
import sys


def write_all(fd: int, data: bytes) -> None:
    view = memoryview(data)
    while view:
        written = os.write(fd, view)
        if written <= 0:
            raise OSError("short run-log write")
        view = view[written:]


def write_receipt(path: Path, nonce: str) -> None:
    temporary = path.with_name(f"{path.name}.tmp-{os.getpid()}")
    fd = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        write_all(fd, f"complete\t{nonce}\n".encode("ascii"))
        os.fsync(fd)
    finally:
        os.close(fd)
    os.replace(temporary, path)
    directory_fd = os.open(path.parent, os.O_RDONLY)
    try:
        os.fsync(directory_fd)
    finally:
        os.close(directory_fd)


def main() -> int:
    if len(sys.argv) != 4 or not sys.argv[3]:
        return 2
    log_path = Path(sys.argv[1])
    receipt_path = Path(sys.argv[2])
    nonce = sys.argv[3]
    log_fd = os.open(log_path, os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
    mirror_open = True
    try:
        while True:
            data = os.read(0, 64 * 1024)
            if not data:
                break
            write_all(log_fd, data)
            if mirror_open:
                try:
                    write_all(1, data)
                except BrokenPipeError:
                    mirror_open = False
        os.fsync(log_fd)
    except Exception as error:  # noqa: BLE001 - the receipt must fail closed.
        try:
            os.write(2, f"run-log sink failed: {error}\n".encode())
        except OSError:
            pass
        return 1
    finally:
        os.close(log_fd)
    try:
        write_receipt(receipt_path, nonce)
    except Exception as error:  # noqa: BLE001 - missing receipt means failure.
        try:
            os.write(2, f"run-log receipt failed: {error}\n".encode())
        except OSError:
            pass
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
