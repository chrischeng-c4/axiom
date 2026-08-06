from __future__ import annotations

from typing import Mapping

from storage_durable.application.durable_replace import (
    DurableReplaceService,
    ReplaceRequest,
)
from storage_durable.domain.failure import FailureKind, StorageFailure
from storage_durable.domain.fsync_policy import FsyncPolicy
from storage_durable.infrastructure.ports import FileSystemPort

class _LocalRecordingFileSystem:
    def __init__(self, files: Mapping[str, bytes] | None = None) -> None:
        self._files: dict[str, bytes] = dict(files) if files is not None else {}
        self._ops: list[tuple[str, str]] = []
        self._fail_once: dict[tuple[str, str], FailureKind] = {}

    def fail_once(self, operation: str, path: str, kind: FailureKind) -> None:
        self._fail_once[(operation, path)] = kind

    def _record(self, operation: str, path: str) -> None:
        self._ops.append((operation, path))
        key = (operation, path)
        if key in self._fail_once:
            kind = self._fail_once.pop(key)
            raise StorageFailure(kind, path)

    def read(self, path: str) -> bytes | None:
        self._record("read", path)
        return self._files.get(path)

    def write(self, path: str, data: bytes) -> None:
        self._record("write", path)
        self._files[path] = bytes(data)

    def append(self, path: str, data: bytes) -> None:
        self._record("append", path)
        existing = self._files.get(path, b"")
        self._files[path] = existing + bytes(data)

    def remove(self, path: str) -> bool:
        self._record("remove", path)
        if path in self._files:
            del self._files[path]
            return True
        return False

    def rename(self, source: str, target: str) -> None:
        self._record("rename", source)
        if source not in self._files:
            raise StorageFailure(FailureKind.NOT_FOUND, source)
        data = self._files.pop(source)
        self._files[target] = data

    def exists(self, path: str) -> bool:
        self._record("exists", path)
        return path in self._files

    def size(self, path: str) -> int | None:
        self._record("size", path)
        data = self._files.get(path)
        return len(data) if data is not None else None

    def truncate(self, path: str, length: int) -> None:
        self._record("truncate", path)
        if path not in self._files:
            raise StorageFailure(FailureKind.NOT_FOUND, path)
        data = self._files[path]
        if length < len(data):
            self._files[path] = data[:length]

    def sync_file(self, path: str) -> None:
        self._record("sync_file", path)

    def sync_directory(self, path: str) -> None:
        self._record("sync_directory", path)

    def list_directory(self, path: str) -> tuple[str, ...]:
        self._record("list_directory", path)
        root = path if path == "/" else path.rstrip("/")
        entries: list[str] = []
        for p in self._files:
            parent = p if p == "/" else p[:p.rfind("/")] if "/" in p else ""
            if parent == root:
                entry = p[p.rfind("/") + 1:] if "/" in p else p
                if entry not in entries:
                    entries.append(entry)
        return tuple(entries)

    def make_directories(self, path: str) -> None:
        self._record("make_directories", path)

MINIMUM_CHECKS = 14

CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX = (
    ("the_recorded_step_sequence_under_the_always_policy_is_the_declared_one", (
        ("make_directories", "var/state"),
        ("remove", "var/state/x.bin.tmp"),
        ("write", "var/state/x.bin.tmp"),
        ("sync_file", "var/state/x.bin.tmp"),
        ("rename", "var/state/x.bin.tmp"),
        ("sync_directory", "var/state"),
    )),
    ("the_recorded_step_sequence_under_the_every_sec_policy_is_the_same_as_always", (
        ("make_directories", "var/state"),
        ("remove", "var/state/x.bin.tmp"),
        ("write", "var/state/x.bin.tmp"),
        ("sync_file", "var/state/x.bin.tmp"),
        ("rename", "var/state/x.bin.tmp"),
        ("sync_directory", "var/state"),
    )),
    ("the_recorded_step_sequence_under_the_interval_policy_is_the_same_as_always", (
        ("make_directories", "var/state"),
        ("remove", "var/state/x.bin.tmp"),
        ("write", "var/state/x.bin.tmp"),
        ("sync_file", "var/state/x.bin.tmp"),
        ("rename", "var/state/x.bin.tmp"),
        ("sync_directory", "var/state"),
    )),
    ("the_recorded_step_sequence_under_the_os_policy_drops_both_forces", (
        ("make_directories", "var/state"),
        ("remove", "var/state/x.bin.tmp"),
        ("write", "var/state/x.bin.tmp"),
        ("rename", "var/state/x.bin.tmp"),
    )),
    ("the_staging_path_is_the_target_path_with_the_declared_suffix_appended", "var/state/x.bin.tmp"),
    ("the_staging_path_stays_in_the_targets_own_directory", "var/state"),
    ("the_payload_lands_at_exactly_the_path_the_caller_named", [112, 97, 121, 108, 111, 97, 100, 45, 97, 108, 112, 104, 97]),
    ("no_staging_file_survives_a_successful_replacement", False),
    ("a_second_replacement_replaces_the_contents_rather_than_extending_them", [112, 97, 121, 108, 111, 97, 100, 45, 98, 101, 116, 97]),
    ("the_parent_directory_is_created_before_the_staging_write", True),
    ("a_target_with_no_directory_component_creates_no_directory", False),
    ("the_file_force_precedes_the_rename", True),
    ("the_directory_force_follows_the_rename", True),
    ("the_step_count_is_the_same_under_every_forcing_policy", (6, 6, 6, 4)),
)

def verify_crash_safe_replacement_behavior() -> dict[str, object]:
    checks = []

    # 1. the_recorded_step_sequence_under_the_always_policy_is_the_declared_one
    exp1 = CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[0][1]
    fs1 = _LocalRecordingFileSystem()
    svc1 = DurableReplaceService(fs1)
    svc1.replace(ReplaceRequest("var/state/x.bin", b"payload-alpha", FsyncPolicy.ALWAYS))
    obs1 = tuple(fs1._ops)
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[0][0],
        "expected": exp1,
        "observed": obs1,
        "passed": obs1 == exp1,
    })

    # 2. the_recorded_step_sequence_under_the_every_sec_policy_is_the_same_as_always
    exp2 = CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[1][1]
    fs2 = _LocalRecordingFileSystem()
    svc2 = DurableReplaceService(fs2)
    svc2.replace(ReplaceRequest("var/state/x.bin", b"payload-alpha", FsyncPolicy.EVERY_SEC))
    obs2 = tuple(fs2._ops)
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[1][0],
        "expected": exp2,
        "observed": obs2,
        "passed": obs2 == exp2,
    })

    # 3. the_recorded_step_sequence_under_the_interval_policy_is_the_same_as_always
    exp3 = CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[2][1]
    fs3 = _LocalRecordingFileSystem()
    svc3 = DurableReplaceService(fs3)
    svc3.replace(ReplaceRequest("var/state/x.bin", b"payload-alpha", FsyncPolicy.INTERVAL))
    obs3 = tuple(fs3._ops)
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[2][0],
        "expected": exp3,
        "observed": obs3,
        "passed": obs3 == exp3,
    })

    # 4. the_recorded_step_sequence_under_the_os_policy_drops_both_forces
    exp4 = CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[3][1]
    fs4 = _LocalRecordingFileSystem()
    svc4 = DurableReplaceService(fs4)
    svc4.replace(ReplaceRequest("var/state/x.bin", b"payload-alpha", FsyncPolicy.OS))
    obs4 = tuple(fs4._ops)
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[3][0],
        "expected": exp4,
        "observed": obs4,
        "passed": obs4 == exp4,
    })

    # 5. the_staging_path_is_the_target_path_with_the_declared_suffix_appended
    exp5 = CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[4][1]
    obs5 = [path for op, path in fs1._ops if op == "write"][0]
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[4][0],
        "expected": exp5,
        "observed": obs5,
        "passed": obs5 == exp5,
    })

    # 6. the_staging_path_stays_in_the_targets_own_directory
    exp6 = CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[5][1]
    staging_file = [path for op, path in fs1._ops if op == "write"][0]
    obs6 = staging_file[:staging_file.rfind("/")]
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[5][0],
        "expected": exp6,
        "observed": obs6,
        "passed": obs6 == exp6,
    })

    # 7. the_payload_lands_at_exactly_the_path_the_caller_named
    exp7 = CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[6][1]
    raw7 = fs1.read("var/state/x.bin")
    obs7 = list(raw7) if raw7 is not None else []
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[6][0],
        "expected": exp7,
        "observed": obs7,
        "passed": obs7 == exp7,
    })

    # 8. no_staging_file_survives_a_successful_replacement
    exp8 = CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[7][1]
    obs8 = fs1.exists("var/state/x.bin.tmp")
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[7][0],
        "expected": exp8,
        "observed": obs8,
        "passed": obs8 == exp8,
    })

    # 9. a_second_replacement_replaces_the_contents_rather_than_extending_them
    exp9 = CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[8][1]
    svc1.replace(ReplaceRequest("var/state/x.bin", b"payload-beta", FsyncPolicy.ALWAYS))
    raw9 = fs1.read("var/state/x.bin")
    obs9 = list(raw9) if raw9 is not None else []
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[8][0],
        "expected": exp9,
        "observed": obs9,
        "passed": obs9 == exp9,
    })

    # 10. the_parent_directory_is_created_before_the_staging_write
    exp10 = CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[9][1]
    mkdir_idx = [i for i, (op, _) in enumerate(fs1._ops) if op == "make_directories"][0]
    write_idx = [i for i, (op, _) in enumerate(fs1._ops) if op == "write"][0]
    obs10 = mkdir_idx < write_idx
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[9][0],
        "expected": exp10,
        "observed": obs10,
        "passed": obs10 == exp10,
    })

    # 11. a_target_with_no_directory_component_creates_no_directory
    exp11 = CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[10][1]
    fs11 = _LocalRecordingFileSystem()
    svc11 = DurableReplaceService(fs11)
    svc11.replace(ReplaceRequest("nodir.bin", b"payload-nodir", FsyncPolicy.ALWAYS))
    obs11 = any(op == "make_directories" for op, _ in fs11._ops)
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[10][0],
        "expected": exp11,
        "observed": obs11,
        "passed": obs11 == exp11,
    })

    # 12. the_file_force_precedes_the_rename
    exp12 = CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[11][1]
    sync_file_idx = [i for i, (op, _) in enumerate(fs1._ops) if op == "sync_file"][0]
    rename_idx = [i for i, (op, _) in enumerate(fs1._ops) if op == "rename"][0]
    obs12 = sync_file_idx < rename_idx
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[11][0],
        "expected": exp12,
        "observed": obs12,
        "passed": obs12 == exp12,
    })

    # 13. the_directory_force_follows_the_rename
    exp13 = CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[12][1]
    sync_dir_idx = [i for i, (op, _) in enumerate(fs1._ops) if op == "sync_directory"][0]
    obs13 = sync_dir_idx > rename_idx
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[12][0],
        "expected": exp13,
        "observed": obs13,
        "passed": obs13 == exp13,
    })

    # 14. the_step_count_is_the_same_under_every_forcing_policy
    exp14 = CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[13][1]
    fs_always = _LocalRecordingFileSystem()
    svc_always = DurableReplaceService(fs_always)
    svc_always.replace(ReplaceRequest("var/state/x.bin", b"payload-alpha", FsyncPolicy.ALWAYS))

    fs_every_sec = _LocalRecordingFileSystem()
    svc_every_sec = DurableReplaceService(fs_every_sec)
    svc_every_sec.replace(ReplaceRequest("var/state/x.bin", b"payload-alpha", FsyncPolicy.EVERY_SEC))

    fs_interval = _LocalRecordingFileSystem()
    svc_interval = DurableReplaceService(fs_interval)
    svc_interval.replace(ReplaceRequest("var/state/x.bin", b"payload-alpha", FsyncPolicy.INTERVAL))

    fs_os = _LocalRecordingFileSystem()
    svc_os = DurableReplaceService(fs_os)
    svc_os.replace(ReplaceRequest("var/state/x.bin", b"payload-alpha", FsyncPolicy.OS))

    obs14 = (len(fs_always._ops), len(fs_every_sec._ops), len(fs_interval._ops), len(fs_os._ops))
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_BEHAVIOR_MATRIX[13][0],
        "expected": exp14,
        "observed": obs14,
        "passed": obs14 == exp14,
    })

    return {
        "case_id": "crash-safe-replacement-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
