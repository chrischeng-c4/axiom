#!/usr/bin/env python3
"""Read process identities from one kernel record per process."""

from __future__ import annotations

import ctypes
import errno
import os
from pathlib import Path
import platform
import re
import sys


def linux_record(pid: int, boot_id: str | None = None) -> tuple[int, int, str, str]:
    try:
        stat_text = Path(f"/proc/{pid}/stat").read_text(encoding="utf-8")
    except FileNotFoundError as error:
        raise ProcessLookupError(pid) from error
    comm_end = stat_text.rfind(")")
    if comm_end < 0:
        raise ValueError("invalid /proc stat record")
    fields_from_state = stat_text[comm_end + 2 :].split()
    if len(fields_from_state) <= 19:
        raise ValueError("short /proc stat record")
    start_ticks = fields_from_state[19]
    if not start_ticks.isdigit():
        raise ValueError("invalid /proc start time")
    if boot_id is None:
        boot_id = Path("/proc/sys/kernel/random/boot_id").read_text(
            encoding="ascii"
        ).strip()
    if not re.fullmatch(r"[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}", boot_id):
        raise ValueError("invalid Linux boot id")
    state = fields_from_state[0]
    if not re.fullmatch(r"[A-Za-z]", state):
        raise ValueError("invalid Linux process state")
    parent_pid = fields_from_state[1]
    process_group = fields_from_state[2]
    if not parent_pid.isdigit() or not process_group.isdigit():
        raise ValueError("invalid Linux process ancestry")
    return (
        int(parent_pid),
        int(process_group),
        f"linux:{boot_id}:{start_ticks}",
        "stopped" if state in ("T", "t") else "running",
    )


def linux_identity(pid: int) -> tuple[str, str]:
    _parent_pid, _process_group, token, state = linux_record(pid)
    return token, state


def linux_snapshot() -> list[tuple[int, int, int, str]]:
    boot_id = Path("/proc/sys/kernel/random/boot_id").read_text(
        encoding="ascii"
    ).strip()
    if not re.fullmatch(r"[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}", boot_id):
        raise ValueError("invalid Linux boot id")
    own_pid = os.getpid()
    records: list[tuple[int, int, int, str]] = []
    for entry in Path("/proc").iterdir():
        if not entry.name.isdigit():
            continue
        pid = int(entry.name)
        if pid == own_pid:
            continue
        try:
            parent_pid, process_group, token, _state = linux_record(pid, boot_id)
        except ProcessLookupError:
            continue
        records.append((pid, parent_pid, process_group, token))
    return records


class ProcBsdInfo(ctypes.Structure):
    _fields_ = [
        ("pbi_flags", ctypes.c_uint32),
        ("pbi_status", ctypes.c_uint32),
        ("pbi_xstatus", ctypes.c_uint32),
        ("pbi_pid", ctypes.c_uint32),
        ("pbi_ppid", ctypes.c_uint32),
        ("pbi_uid", ctypes.c_uint32),
        ("pbi_gid", ctypes.c_uint32),
        ("pbi_ruid", ctypes.c_uint32),
        ("pbi_rgid", ctypes.c_uint32),
        ("pbi_svuid", ctypes.c_uint32),
        ("pbi_svgid", ctypes.c_uint32),
        ("rfu_1", ctypes.c_uint32),
        ("pbi_comm", ctypes.c_char * 16),
        ("pbi_name", ctypes.c_char * 32),
        ("pbi_nfiles", ctypes.c_uint32),
        ("pbi_pgid", ctypes.c_uint32),
        ("pbi_pjobc", ctypes.c_uint32),
        ("e_tdev", ctypes.c_uint32),
        ("e_tpgid", ctypes.c_uint32),
        ("pbi_nice", ctypes.c_int32),
        ("pbi_start_tvsec", ctypes.c_uint64),
        ("pbi_start_tvusec", ctypes.c_uint64),
    ]


def darwin_libproc() -> ctypes.CDLL:
    libproc = ctypes.CDLL("/usr/lib/libproc.dylib", use_errno=True)
    libproc.proc_pidinfo.argtypes = [
        ctypes.c_int,
        ctypes.c_int,
        ctypes.c_uint64,
        ctypes.c_void_p,
        ctypes.c_int,
    ]
    libproc.proc_pidinfo.restype = ctypes.c_int
    return libproc


def darwin_record(
    libproc: ctypes.CDLL, pid: int
) -> tuple[int, int, str, str]:
    info = ProcBsdInfo()
    result = libproc.proc_pidinfo(
        pid,
        3,  # PROC_PIDTBSDINFO
        0,
        ctypes.byref(info),
        ctypes.sizeof(info),
    )
    if result != ctypes.sizeof(info) or info.pbi_pid != pid:
        error_number = ctypes.get_errno()
        if error_number in (errno.EPERM, errno.EACCES):
            raise PermissionError(error_number, "proc_pidinfo denied", pid)
        if error_number == 0:
            try:
                os.kill(pid, 0)
            except ProcessLookupError:
                raise
            except PermissionError:
                raise
            raise OSError(errno.EIO, "proc_pidinfo returned no identity", pid)
        if error_number != errno.ESRCH:
            raise OSError(error_number, "proc_pidinfo failed", pid)
        raise ProcessLookupError(pid)
    if info.pbi_start_tvsec == 0:
        raise ValueError("missing Darwin process start time")
    return (
        int(info.pbi_ppid),
        int(info.pbi_pgid),
        f"darwin:{info.pbi_start_tvsec}:{info.pbi_start_tvusec}",
        "stopped" if info.pbi_status == 4 else "running",
    )


def darwin_identity(pid: int) -> tuple[str, str]:
    _parent_pid, _process_group, token, state = darwin_record(
        darwin_libproc(), pid
    )
    return token, state


def darwin_snapshot() -> list[tuple[int, int, int, str]]:
    libproc = darwin_libproc()
    libproc.proc_listallpids.argtypes = [ctypes.c_void_p, ctypes.c_int]
    libproc.proc_listallpids.restype = ctypes.c_int
    count = libproc.proc_listallpids(None, 0)
    if count <= 0:
        raise OSError(errno.EIO, "proc_listallpids returned no process count")
    # Processes can appear between the sizing and data calls. Keep spare room
    # so that a valid concurrent fork does not truncate the snapshot.
    capacity = count + 1024
    pids = (ctypes.c_int * capacity)()
    result = libproc.proc_listallpids(pids, ctypes.sizeof(pids))
    if result < 0 or result > capacity:
        raise OSError(errno.EIO, "proc_listallpids returned an invalid count")
    own_pid = os.getpid()
    records: list[tuple[int, int, int, str]] = []
    for pid in pids[:result]:
        if pid <= 0 or pid == own_pid:
            continue
        try:
            parent_pid, process_group, token, _state = darwin_record(libproc, pid)
        except (ProcessLookupError, PermissionError):
            # macOS can list protected system processes but deny their BSD
            # record. They cannot be descendants of this unprivileged run.
            continue
        records.append((pid, parent_pid, process_group, token))
    return records


def process_snapshot(system: str) -> list[tuple[int, int, int, str]]:
    if system == "Linux":
        return linux_snapshot()
    if system == "Darwin":
        return darwin_snapshot()
    raise OSError(errno.ENOTSUP, "unsupported process snapshot platform")


def main() -> int:
    snapshot_mode = len(sys.argv) == 2 and sys.argv[1] == "--snapshot"
    if snapshot_mode:
        try:
            records = process_snapshot(platform.system())
        except (OSError, ValueError):
            return 4
        for pid, parent_pid, process_group, token in sorted(records):
            print(f"{pid}\t{parent_pid}\t{process_group}\t{token}")
        return 0
    status_mode = len(sys.argv) == 4 and sys.argv[1] == "--status"
    if status_mode:
        pid_argument = sys.argv[2]
        expected_token = sys.argv[3]
    elif len(sys.argv) == 2:
        pid_argument = sys.argv[1]
        expected_token = ""
    else:
        return 2
    if not re.fullmatch(r"[1-9][0-9]*", pid_argument):
        return 2
    pid = int(pid_argument)
    try:
        system = platform.system()
        if system == "Linux":
            token, state = linux_identity(pid)
        elif system == "Darwin":
            token, state = darwin_identity(pid)
        else:
            return 3
    except ProcessLookupError:
        return 1
    except (OSError, ValueError):
        return 4
    if status_mode:
        if token != expected_token:
            return 1
        print(state)
    else:
        print(token)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
