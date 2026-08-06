from __future__ import annotations

from typing import Mapping

from storage_durable.application.durable_replace import DurableReplaceService
from storage_durable.application.snapshot_store import (
    SnapshotStoreConfig,
    SnapshotStoreService,
)
from storage_durable.domain.failure import FailureKind, StorageFailure
from storage_durable.domain.fsync_policy import FsyncPolicy
from storage_durable.domain.snapshot_name import parse_name
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

SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX = (
    ("a_non_numeric_sequence_is_not_a_snapshot_of_this_store", None),
    ("a_negative_sequence_is_not_a_snapshot_of_this_store", None),
    ("a_leading_zero_sequence_is_not_a_snapshot_of_this_store", None),
    ("a_sequence_with_surrounding_whitespace_is_not_a_snapshot_of_this_store", None),
    ("a_non_ascii_digit_sequence_is_not_a_snapshot_of_this_store", None),
    ("another_stores_prefix_is_not_a_snapshot_of_this_store", None),
    ("another_extension_is_not_a_snapshot_of_this_store", None),
    ("a_name_with_no_separator_is_not_a_snapshot_of_this_store", None),
    ("an_empty_sequence_is_not_a_snapshot_of_this_store", None),
    ("the_listing_over_the_seeded_directory_is_exactly_the_legitimate_sequences", (1, 2)),
    ("the_latest_is_never_a_foreign_file", [108, 101, 103, 45, 50]),
    ("pruning_removes_no_foreign_file", True),
    ("pruning_removes_at_most_the_listed_count_minus_the_retention_count", True),
    ("the_removed_count_is_computed_from_the_legitimate_snapshots_alone", 1),
)

def verify_sequence_ordered_snapshots_security() -> dict[str, object]:
    checks = []
    cfg = SnapshotStoreConfig(root="var/snaps", prefix="snap", extension="bin", policy=FsyncPolicy.ALWAYS)

    # 1. a_non_numeric_sequence_is_not_a_snapshot_of_this_store
    exp1 = SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[0][1]
    obs1 = parse_name("snap-x.bin", "snap", "bin")
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[0][0],
        "expected": exp1,
        "observed": obs1,
        "passed": obs1 == exp1,
    })

    # 2. a_negative_sequence_is_not_a_snapshot_of_this_store
    exp2 = SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[1][1]
    obs2 = parse_name("snap--1.bin", "snap", "bin")
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[1][0],
        "expected": exp2,
        "observed": obs2,
        "passed": obs2 == exp2,
    })

    # 3. a_leading_zero_sequence_is_not_a_snapshot_of_this_store
    exp3 = SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[2][1]
    obs3 = parse_name("snap-007.bin", "snap", "bin")
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[2][0],
        "expected": exp3,
        "observed": obs3,
        "passed": obs3 == exp3,
    })

    # 4. a_sequence_with_surrounding_whitespace_is_not_a_snapshot_of_this_store
    exp4 = SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[3][1]
    obs4 = parse_name("snap- 7.bin", "snap", "bin")
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[3][0],
        "expected": exp4,
        "observed": obs4,
        "passed": obs4 == exp4,
    })

    # 5. a_non_ascii_digit_sequence_is_not_a_snapshot_of_this_store
    exp5 = SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[4][1]
    obs5 = parse_name("snap-².bin", "snap", "bin")
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[4][0],
        "expected": exp5,
        "observed": obs5,
        "passed": obs5 == exp5,
    })

    # 6. another_stores_prefix_is_not_a_snapshot_of_this_store
    exp6 = SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[5][1]
    obs6 = parse_name("other-7.bin", "snap", "bin")
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[5][0],
        "expected": exp6,
        "observed": obs6,
        "passed": obs6 == exp6,
    })

    # 7. another_extension_is_not_a_snapshot_of_this_store
    exp7 = SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[6][1]
    obs7 = parse_name("snap-7.dat", "snap", "bin")
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[6][0],
        "expected": exp7,
        "observed": obs7,
        "passed": obs7 == exp7,
    })

    # 8. a_name_with_no_separator_is_not_a_snapshot_of_this_store
    exp8 = SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[7][1]
    obs8 = parse_name("snap7.bin", "snap", "bin")
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[7][0],
        "expected": exp8,
        "observed": obs8,
        "passed": obs8 == exp8,
    })

    # 9. an_empty_sequence_is_not_a_snapshot_of_this_store
    exp9 = SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[8][1]
    obs9 = parse_name("snap-.bin", "snap", "bin")
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[8][0],
        "expected": exp9,
        "observed": obs9,
        "passed": obs9 == exp9,
    })

    foreign_files = {
        "var/snaps/snap-x.bin": b"f1",
        "var/snaps/snap--1.bin": b"f2",
        "var/snaps/snap-007.bin": b"f3",
        "var/snaps/snap- 7.bin": b"f4",
        "var/snaps/snap-².bin": b"f5",
        "var/snaps/other-7.bin": b"f6",
        "var/snaps/snap-7.dat": b"f7",
        "var/snaps/snap7.bin": b"f8",
        "var/snaps/snap-.bin": b"f9",
    }
    fs10 = _LocalRecordingFileSystem(foreign_files)
    rep10 = DurableReplaceService(fs10)
    svc10 = SnapshotStoreService(fs10, rep10)
    svc10.save(cfg, 1, b"leg-1")
    svc10.save(cfg, 2, b"leg-2")

    # 10. the_listing_over_the_seeded_directory_is_exactly_the_legitimate_sequences
    exp10 = SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[9][1]
    obs10 = tuple(e.seq for e in svc10.entries(cfg))
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[9][0],
        "expected": exp10,
        "observed": obs10,
        "passed": obs10 == exp10,
    })

    # 11. the_latest_is_never_a_foreign_file
    exp11 = SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[10][1]
    raw11 = svc10.load_latest(cfg)
    obs11 = list(raw11) if raw11 is not None else []
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[10][0],
        "expected": exp11,
        "observed": obs11,
        "passed": obs11 == exp11,
    })

    # 12. pruning_removes_no_foreign_file
    exp12 = SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[11][1]
    rem_cnt12 = svc10.prune(cfg, keep=1)
    obs12 = all(fs10.exists(f_path) for f_path in foreign_files)
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[11][0],
        "expected": exp12,
        "observed": obs12,
        "passed": obs12 == exp12,
    })

    # 13. pruning_removes_at_most_the_listed_count_minus_the_retention_count
    exp13 = SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[12][1]
    obs13 = rem_cnt12 <= (2 - 1)
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[12][0],
        "expected": exp13,
        "observed": obs13,
        "passed": obs13 == exp13,
    })

    # 14. the_removed_count_is_computed_from_the_legitimate_snapshots_alone
    exp14 = SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[13][1]
    obs14 = rem_cnt12
    checks.append({
        "name": SEQUENCE_ORDERED_SNAPSHOTS_SECURITY_MATRIX[13][0],
        "expected": exp14,
        "observed": obs14,
        "passed": obs14 == exp14,
    })

    return {
        "case_id": "sequence-ordered-snapshots-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
