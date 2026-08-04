from __future__ import annotations

from typing import Mapping

from storage_durable.application.durable_replace import (
    DurableReplaceService,
    ReplaceRequest,
)
from storage_durable.domain.failure import (
    FailureKind,
    StorageFailure,
    is_tolerated_directory_sync_failure,
)
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

MINIMUM_CHECKS = 13

CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX = (
    ("a_failure_at_the_staging_write_leaves_the_previous_version_complete", [111, 108, 100, 45, 112, 97, 121, 108, 111, 97, 100, 45, 49, 50, 51]),
    ("a_failure_at_the_file_force_leaves_the_previous_version_complete", [111, 108, 100, 45, 112, 97, 121, 108, 111, 97, 100, 45, 49, 50, 51]),
    ("a_failure_at_the_rename_leaves_the_previous_version_complete", [111, 108, 100, 45, 112, 97, 121, 108, 111, 97, 100, 45, 49, 50, 51]),
    ("no_failing_row_ever_publishes_the_new_payload_at_the_target", False),
    ("the_stale_staging_file_is_discarded_before_the_new_write_begins", True),
    ("the_stale_bytes_are_never_readable_at_the_target_path", False),
    ("no_stale_staging_residue_survives_a_failed_replacement", False),
    ("the_tolerated_directory_force_failure_classes_are_exactly_the_contracts_set", ("other", "permission-denied", "unsupported")),
    ("a_not_found_directory_force_failure_propagates", True),
    ("an_io_directory_force_failure_propagates", True),
    ("a_permission_denied_directory_force_failure_is_absorbed", False),
    ("an_absorbed_directory_force_failure_still_leaves_the_new_payload_published", [110, 101, 119, 45, 112, 97, 121, 108, 111, 97, 100, 45, 52, 53, 54]),
    ("the_target_is_published_by_a_rename_and_never_by_a_direct_write", False),
)

def verify_crash_safe_replacement_security() -> dict[str, object]:
    checks = []
    old_bytes = b"old-payload-123"
    new_bytes = b"new-payload-456"
    stale_bytes = b"stale-staging-789"

    # 1. a_failure_at_the_staging_write_leaves_the_previous_version_complete
    exp1 = CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[0][1]
    fs1 = _LocalRecordingFileSystem({"var/state/x.bin": old_bytes})
    fs1.fail_once("write", "var/state/x.bin.tmp", FailureKind.IO)
    svc1 = DurableReplaceService(fs1)
    try:
        svc1.replace(ReplaceRequest("var/state/x.bin", new_bytes, FsyncPolicy.ALWAYS))
    except StorageFailure:
        pass
    raw1 = fs1.read("var/state/x.bin")
    obs1 = list(raw1) if raw1 is not None else []
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[0][0],
        "expected": exp1,
        "observed": obs1,
        "passed": obs1 == exp1,
    })

    # 2. a_failure_at_the_file_force_leaves_the_previous_version_complete
    exp2 = CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[1][1]
    fs2 = _LocalRecordingFileSystem({"var/state/x.bin": old_bytes})
    fs2.fail_once("sync_file", "var/state/x.bin.tmp", FailureKind.IO)
    svc2 = DurableReplaceService(fs2)
    try:
        svc2.replace(ReplaceRequest("var/state/x.bin", new_bytes, FsyncPolicy.ALWAYS))
    except StorageFailure:
        pass
    raw2 = fs2.read("var/state/x.bin")
    obs2 = list(raw2) if raw2 is not None else []
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[1][0],
        "expected": exp2,
        "observed": obs2,
        "passed": obs2 == exp2,
    })

    # 3. a_failure_at_the_rename_leaves_the_previous_version_complete
    exp3 = CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[2][1]
    fs3 = _LocalRecordingFileSystem({"var/state/x.bin": old_bytes})
    fs3.fail_once("rename", "var/state/x.bin.tmp", FailureKind.IO)
    svc3 = DurableReplaceService(fs3)
    try:
        svc3.replace(ReplaceRequest("var/state/x.bin", new_bytes, FsyncPolicy.ALWAYS))
    except StorageFailure:
        pass
    raw3 = fs3.read("var/state/x.bin")
    obs3 = list(raw3) if raw3 is not None else []
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[2][0],
        "expected": exp3,
        "observed": obs3,
        "passed": obs3 == exp3,
    })

    # 4. no_failing_row_ever_publishes_the_new_payload_at_the_target
    exp4 = CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[3][1]
    obs4 = any(raw == new_bytes for raw in [raw1, raw2, raw3])
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[3][0],
        "expected": exp4,
        "observed": obs4,
        "passed": obs4 == exp4,
    })

    # 5. the_stale_staging_file_is_discarded_before_the_new_write_begins
    exp5 = CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[4][1]
    fs5 = _LocalRecordingFileSystem({"var/state/x.bin.tmp": stale_bytes})
    svc5 = DurableReplaceService(fs5)
    svc5.replace(ReplaceRequest("var/state/x.bin", new_bytes, FsyncPolicy.ALWAYS))
    rem_idx = [i for i, (op, p) in enumerate(fs5._ops) if op == "remove" and p == "var/state/x.bin.tmp"][0]
    wrt_idx = [i for i, (op, p) in enumerate(fs5._ops) if op == "write" and p == "var/state/x.bin.tmp"][0]
    obs5 = rem_idx < wrt_idx
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[4][0],
        "expected": exp5,
        "observed": obs5,
        "passed": obs5 == exp5,
    })

    # 6. the_stale_bytes_are_never_readable_at_the_target_path
    exp6 = CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[5][1]
    obs6 = fs5.read("var/state/x.bin") == stale_bytes
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[5][0],
        "expected": exp6,
        "observed": obs6,
        "passed": obs6 == exp6,
    })

    # 7. no_stale_staging_residue_survives_a_failed_replacement
    exp7 = CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[6][1]
    fs7 = _LocalRecordingFileSystem({"var/state/x.bin.tmp": stale_bytes})
    fs7.fail_once("write", "var/state/x.bin.tmp", FailureKind.IO)
    svc7 = DurableReplaceService(fs7)
    try:
        svc7.replace(ReplaceRequest("var/state/x.bin", new_bytes, FsyncPolicy.ALWAYS))
    except StorageFailure:
        pass
    obs7 = fs7.read("var/state/x.bin.tmp") == stale_bytes
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[6][0],
        "expected": exp7,
        "observed": obs7,
        "passed": obs7 == exp7,
    })

    # 8. the_tolerated_directory_force_failure_classes_are_exactly_the_contracts_set
    exp8 = CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[7][1]
    tolerated = [k.value for k in FailureKind if is_tolerated_directory_sync_failure(k)]
    obs8 = tuple(sorted(tolerated))
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[7][0],
        "expected": exp8,
        "observed": obs8,
        "passed": obs8 == exp8,
    })

    # 9. a_not_found_directory_force_failure_propagates
    exp9 = CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[8][1]
    fs9 = _LocalRecordingFileSystem()
    fs9.fail_once("sync_directory", "var/state", FailureKind.NOT_FOUND)
    svc9 = DurableReplaceService(fs9)
    raised9 = False
    try:
        svc9.replace(ReplaceRequest("var/state/x.bin", new_bytes, FsyncPolicy.ALWAYS))
    except StorageFailure:
        raised9 = True
    obs9 = raised9
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[8][0],
        "expected": exp9,
        "observed": obs9,
        "passed": obs9 == exp9,
    })

    # 10. an_io_directory_force_failure_propagates
    exp10 = CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[9][1]
    fs10 = _LocalRecordingFileSystem()
    fs10.fail_once("sync_directory", "var/state", FailureKind.IO)
    svc10 = DurableReplaceService(fs10)
    raised10 = False
    try:
        svc10.replace(ReplaceRequest("var/state/x.bin", new_bytes, FsyncPolicy.ALWAYS))
    except StorageFailure:
        raised10 = True
    obs10 = raised10
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[9][0],
        "expected": exp10,
        "observed": obs10,
        "passed": obs10 == exp10,
    })

    # 11. a_permission_denied_directory_force_failure_is_absorbed
    exp11 = CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[10][1]
    fs11 = _LocalRecordingFileSystem()
    fs11.fail_once("sync_directory", "var/state", FailureKind.PERMISSION_DENIED)
    svc11 = DurableReplaceService(fs11)
    raised11 = False
    try:
        svc11.replace(ReplaceRequest("var/state/x.bin", new_bytes, FsyncPolicy.ALWAYS))
    except Exception:
        raised11 = True
    obs11 = raised11
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[10][0],
        "expected": exp11,
        "observed": obs11,
        "passed": obs11 == exp11,
    })

    # 12. an_absorbed_directory_force_failure_still_leaves_the_new_payload_published
    exp12 = CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[11][1]
    raw12 = fs11.read("var/state/x.bin")
    obs12 = list(raw12) if raw12 is not None else []
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[11][0],
        "expected": exp12,
        "observed": obs12,
        "passed": obs12 == exp12,
    })

    # 13. the_target_is_published_by_a_rename_and_never_by_a_direct_write
    exp13 = CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[12][1]
    obs13 = any(op == "write" and p == "var/state/x.bin" for op, p in fs5._ops)
    checks.append({
        "name": CRASH_SAFE_REPLACEMENT_SECURITY_MATRIX[12][0],
        "expected": exp13,
        "observed": obs13,
        "passed": obs13 == exp13,
    })

    return {
        "case_id": "crash-safe-replacement-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
