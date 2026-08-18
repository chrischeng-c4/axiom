from __future__ import annotations

from typing import Mapping

from storage_durable.application.durable_replace import DurableReplaceService
from storage_durable.application.snapshot_store import (
    SnapshotStoreConfig,
    SnapshotStoreService,
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

MINIMUM_CHECKS = 13

SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX = (
    ("the_listing_is_ascending_by_parsed_sequence_not_by_name", (9, 10)),
    ("the_listing_names_are_the_rendered_names_for_those_sequences", ("snap-9.bin", "snap-10.bin")),
    ("the_listing_is_independent_of_the_order_the_snapshots_were_written", True),
    ("the_latest_payload_is_the_one_written_for_the_highest_sequence", [112, 97, 121, 108, 111, 97, 100, 45, 49, 48]),
    ("the_latest_is_not_the_last_snapshot_written", True),
    ("the_latest_of_an_empty_store_is_absent_rather_than_empty", None),
    ("a_saved_snapshot_lands_at_the_rendered_name_under_the_stores_root", "var/snaps/snap-7.bin"),
    ("pruning_below_the_stored_count_keeps_the_highest_sequences", (3,)),
    ("pruning_below_the_stored_count_reports_the_number_it_removed", 2),
    ("pruning_at_the_stored_count_removes_nothing", 0),
    ("pruning_above_the_stored_count_removes_nothing", 0),
    ("pruning_to_zero_leaves_no_snapshot_of_this_store", 0),
    ("pruning_reports_zero_when_it_removed_nothing", 0),
)

def verify_sequence_ordered_snapshots_behavior() -> dict[str, object]:
    checks = []
    cfg = SnapshotStoreConfig(root="var/snaps", prefix="snap", extension="bin", policy=FsyncPolicy.ALWAYS)

    # Store 1: write 9 then 10
    fs1 = _LocalRecordingFileSystem()
    rep1 = DurableReplaceService(fs1)
    svc1 = SnapshotStoreService(fs1, rep1)
    svc1.save(cfg, 9, b"payload-9")
    svc1.save(cfg, 10, b"payload-10")
    entries1 = svc1.entries(cfg)

    # Store 2: write 10 then 9
    fs2 = _LocalRecordingFileSystem()
    rep2 = DurableReplaceService(fs2)
    svc2 = SnapshotStoreService(fs2, rep2)
    svc2.save(cfg, 10, b"payload-10")
    svc2.save(cfg, 9, b"payload-9")
    entries2 = svc2.entries(cfg)

    # 1. the_listing_is_ascending_by_parsed_sequence_not_by_name
    exp1 = SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[0][1]
    obs1 = tuple(e.seq for e in entries1)
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[0][0],
        "expected": exp1,
        "observed": obs1,
        "passed": obs1 == exp1,
    })

    # 2. the_listing_names_are_the_rendered_names_for_those_sequences
    exp2 = SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[1][1]
    obs2 = tuple(e.name for e in entries1)
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[1][0],
        "expected": exp2,
        "observed": obs2,
        "passed": obs2 == exp2,
    })

    # 3. the_listing_is_independent_of_the_order_the_snapshots_were_written
    exp3 = SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[2][1]
    obs3 = entries1 == entries2
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[2][0],
        "expected": exp3,
        "observed": obs3,
        "passed": obs3 == exp3,
    })

    # 4. the_latest_payload_is_the_one_written_for_the_highest_sequence
    exp4 = SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[3][1]
    raw4 = svc1.load_latest(cfg)
    obs4 = list(raw4) if raw4 is not None else []
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[3][0],
        "expected": exp4,
        "observed": obs4,
        "passed": obs4 == exp4,
    })

    # 5. the_latest_is_not_the_last_snapshot_written
    exp5 = SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[4][1]
    raw5_latest = svc2.load_latest(cfg)
    obs5 = (raw5_latest == b"payload-10")
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[4][0],
        "expected": exp5,
        "observed": obs5,
        "passed": obs5 == exp5,
    })

    # 6. the_latest_of_an_empty_store_is_absent_rather_than_empty
    exp6 = SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[5][1]
    fs6 = _LocalRecordingFileSystem()
    rep6 = DurableReplaceService(fs6)
    svc6 = SnapshotStoreService(fs6, rep6)
    obs6 = svc6.load_latest(cfg)
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[5][0],
        "expected": exp6,
        "observed": obs6,
        "passed": obs6 == exp6,
    })

    # 7. a_saved_snapshot_lands_at_the_rendered_name_under_the_stores_root
    exp7 = SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[6][1]
    fs7 = _LocalRecordingFileSystem()
    rep7 = DurableReplaceService(fs7)
    svc7 = SnapshotStoreService(fs7, rep7)
    saved_path = svc7.save(cfg, 7, b"payload-7")
    obs7 = saved_path
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[6][0],
        "expected": exp7,
        "observed": obs7,
        "passed": obs7 == exp7,
    })

    # Store 8: 1, 2, 3
    fs8 = _LocalRecordingFileSystem()
    rep8 = DurableReplaceService(fs8)
    svc8 = SnapshotStoreService(fs8, rep8)
    svc8.save(cfg, 1, b"s1")
    svc8.save(cfg, 2, b"s2")
    svc8.save(cfg, 3, b"s3")

    # 8. pruning_below_the_stored_count_keeps_the_highest_sequences
    exp8 = SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[7][1]
    rem_cnt8 = svc8.prune(cfg, keep=1)
    obs8 = tuple(e.seq for e in svc8.entries(cfg))
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[7][0],
        "expected": exp8,
        "observed": obs8,
        "passed": obs8 == exp8,
    })

    # 9. pruning_below_the_stored_count_reports_the_number_it_removed
    exp9 = SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[8][1]
    obs9 = rem_cnt8
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[8][0],
        "expected": exp9,
        "observed": obs9,
        "passed": obs9 == exp9,
    })

    # Store 10: 1, 2, 3
    fs10 = _LocalRecordingFileSystem()
    rep10 = DurableReplaceService(fs10)
    svc10 = SnapshotStoreService(fs10, rep10)
    svc10.save(cfg, 1, b"s1")
    svc10.save(cfg, 2, b"s2")
    svc10.save(cfg, 3, b"s3")

    # 10. pruning_at_the_stored_count_removes_nothing
    exp10 = SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[9][1]
    obs10 = svc10.prune(cfg, keep=3)
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[9][0],
        "expected": exp10,
        "observed": obs10,
        "passed": obs10 == exp10,
    })

    # 11. pruning_above_the_stored_count_removes_nothing
    exp11 = SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[10][1]
    obs11 = svc10.prune(cfg, keep=5)
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[10][0],
        "expected": exp11,
        "observed": obs11,
        "passed": obs11 == exp11,
    })

    # 12. pruning_to_zero_leaves_no_snapshot_of_this_store
    exp12 = SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[11][1]
    svc10.prune(cfg, keep=0)
    obs12 = len(svc10.entries(cfg))
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[11][0],
        "expected": exp12,
        "observed": obs12,
        "passed": obs12 == exp12,
    })

    # 13. pruning_reports_zero_when_it_removed_nothing
    exp13 = SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[12][1]
    fs13 = _LocalRecordingFileSystem()
    rep13 = DurableReplaceService(fs13)
    svc13 = SnapshotStoreService(fs13, rep13)
    svc13.save(cfg, 1, b"s1")
    obs13 = svc13.prune(cfg, keep=1)
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_BEHAVIOR_MATRIX[12][0],
        "expected": exp13,
        "observed": obs13,
        "passed": obs13 == exp13,
    })

    return {
        "case_id": "sequence-ordered-snapshots-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
