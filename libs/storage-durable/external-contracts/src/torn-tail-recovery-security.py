from __future__ import annotations

import time
from typing import Mapping
import zlib

from storage_durable.application.framed_log import FramedLogService
from storage_durable.domain.failure import FailureKind, StorageFailure
from storage_durable.domain.frame import (
    HEADER_LENGTH,
    encode_frame,
    encode_header,
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

MINIMUM_CHECKS = 14

TORN_TAIL_RECOVERY_SECURITY_MATRIX = (
    ("a_header_cut_short_ends_the_log_at_the_good_end", 66),
    ("a_declared_length_past_end_of_file_ends_the_log_at_the_good_end", 66),
    ("an_enormous_declared_length_ends_the_log_at_the_good_end", 66),
    ("a_payload_flipped_after_its_checksum_ends_the_log_at_the_good_end", 66),
    ("every_damage_class_recovers_exactly_the_good_prefix_payloads", True),
    ("no_damaged_byte_appears_in_any_returned_payload", False),
    ("the_rejection_reported_names_the_damage_class_that_was_planted", ("header-truncated", "payload-truncated", "payload-truncated", "checksum-mismatch")),
    ("recovery_shortens_the_file_to_the_good_end", 66),
    ("recovery_never_lengthens_the_file", True),
    ("recovery_writes_no_bytes_of_its_own", False),
    ("an_undamaged_log_is_not_truncated_at_all", False),
    ("an_enormous_declared_length_is_refused_without_a_long_pause", True),
    ("an_empty_log_recovers_to_an_empty_frame_list_without_truncating", (0, 0, False)),
    ("the_recovered_frame_count_is_the_same_for_every_damage_class", (3, 3, 3, 3)),
)

def verify_torn_tail_recovery_security() -> dict[str, object]:
    checks = []

    f1 = encode_frame(1, b"frame1")
    f2 = encode_frame(2, b"frame2")
    f3 = encode_frame(3, b"frame3")
    good_buf = f1 + f2 + f3
    good_end = len(good_buf)  # 66

    # Damage 1: header cut short (5 bytes trash)
    d1_buf = good_buf + b"trash"
    # Damage 2: declared length 10 past end (file has 4 payload bytes)
    d2_buf = good_buf + encode_header(4, 10, zlib.crc32(b"1234567890") & 0xFFFFFFFF) + b"1234"
    # Damage 3: enormous declared length 0xFFFFFFFF over 4 bytes payload
    d3_buf = good_buf + encode_header(4, 0xFFFFFFFF, 12345) + b"1234"
    # Damage 4: payload byte flipped
    hdr4 = encode_header(4, 5, zlib.crc32(b"hello") & 0xFFFFFFFF)
    d4_buf = good_buf + hdr4 + b"hallo"

    # 1. a_header_cut_short_ends_the_log_at_the_good_end
    exp1 = TORN_TAIL_RECOVERY_SECURITY_MATRIX[0][1]
    fs1 = _LocalRecordingFileSystem({"var/log/app.log": d1_buf})
    svc1 = FramedLogService(fs1)
    rec1 = svc1.open_for_append("var/log/app.log", FsyncPolicy.ALWAYS)
    obs1 = rec1.good_end
    checks.append({
        "name": TORN_TAIL_RECOVERY_SECURITY_MATRIX[0][0],
        "expected": exp1,
        "observed": obs1,
        "passed": obs1 == exp1,
    })

    # 2. a_declared_length_past_end_of_file_ends_the_log_at_the_good_end
    exp2 = TORN_TAIL_RECOVERY_SECURITY_MATRIX[1][1]
    fs2 = _LocalRecordingFileSystem({"var/log/app.log": d2_buf})
    svc2 = FramedLogService(fs2)
    rec2 = svc2.open_for_append("var/log/app.log", FsyncPolicy.ALWAYS)
    obs2 = rec2.good_end
    checks.append({
        "name": TORN_TAIL_RECOVERY_SECURITY_MATRIX[1][0],
        "expected": exp2,
        "observed": obs2,
        "passed": obs2 == exp2,
    })

    # 3. an_enormous_declared_length_ends_the_log_at_the_good_end
    exp3 = TORN_TAIL_RECOVERY_SECURITY_MATRIX[2][1]
    t_start = time.perf_counter()
    fs3 = _LocalRecordingFileSystem({"var/log/app.log": d3_buf})
    svc3 = FramedLogService(fs3)
    rec3 = svc3.open_for_append("var/log/app.log", FsyncPolicy.ALWAYS)
    t_elapsed = time.perf_counter() - t_start
    obs3 = rec3.good_end
    checks.append({
        "name": TORN_TAIL_RECOVERY_SECURITY_MATRIX[2][0],
        "expected": exp3,
        "observed": obs3,
        "passed": obs3 == exp3,
    })

    # 4. a_payload_flipped_after_its_checksum_ends_the_log_at_the_good_end
    exp4 = TORN_TAIL_RECOVERY_SECURITY_MATRIX[3][1]
    fs4 = _LocalRecordingFileSystem({"var/log/app.log": d4_buf})
    svc4 = FramedLogService(fs4)
    rec4 = svc4.open_for_append("var/log/app.log", FsyncPolicy.ALWAYS)
    obs4 = rec4.good_end
    checks.append({
        "name": TORN_TAIL_RECOVERY_SECURITY_MATRIX[3][0],
        "expected": exp4,
        "observed": obs4,
        "passed": obs4 == exp4,
    })

    # 5. every_damage_class_recovers_exactly_the_good_prefix_payloads
    exp5 = TORN_TAIL_RECOVERY_SECURITY_MATRIX[4][1]
    expected_payloads = [b"frame1", b"frame2", b"frame3"]
    obs5 = all([f.payload for f in r.frames] == expected_payloads for r in [rec1, rec2, rec3, rec4])
    checks.append({
        "name": TORN_TAIL_RECOVERY_SECURITY_MATRIX[4][0],
        "expected": exp5,
        "observed": obs5,
        "passed": obs5 == exp5,
    })

    # 6. no_damaged_byte_appears_in_any_returned_payload
    exp6 = TORN_TAIL_RECOVERY_SECURITY_MATRIX[5][1]
    obs6 = any(b"trash" in f.payload or b"hallo" in f.payload for r in [rec1, rec2, rec3, rec4] for f in r.frames)
    checks.append({
        "name": TORN_TAIL_RECOVERY_SECURITY_MATRIX[5][0],
        "expected": exp6,
        "observed": obs6,
        "passed": obs6 == exp6,
    })

    # 7. the_rejection_reported_names_the_damage_class_that_was_planted
    exp7 = TORN_TAIL_RECOVERY_SECURITY_MATRIX[6][1]
    obs7 = (
        str(rec1.rejection.value) if rec1.rejection else "",
        str(rec2.rejection.value) if rec2.rejection else "",
        str(rec3.rejection.value) if rec3.rejection else "",
        str(rec4.rejection.value) if rec4.rejection else "",
    )
    checks.append({
        "name": TORN_TAIL_RECOVERY_SECURITY_MATRIX[6][0],
        "expected": exp7,
        "observed": obs7,
        "passed": obs7 == exp7,
    })

    # 8. recovery_shortens_the_file_to_the_good_end
    exp8 = TORN_TAIL_RECOVERY_SECURITY_MATRIX[7][1]
    obs8 = fs1.size("var/log/app.log")
    checks.append({
        "name": TORN_TAIL_RECOVERY_SECURITY_MATRIX[7][0],
        "expected": exp8,
        "observed": obs8,
        "passed": obs8 == exp8,
    })

    # 9. recovery_never_lengthens_the_file
    exp9 = TORN_TAIL_RECOVERY_SECURITY_MATRIX[8][1]
    obs9 = all(fs.size("var/log/app.log") <= len(buf) for fs, buf in [(fs1, d1_buf), (fs2, d2_buf), (fs3, d3_buf), (fs4, d4_buf)])
    checks.append({
        "name": TORN_TAIL_RECOVERY_SECURITY_MATRIX[8][0],
        "expected": exp9,
        "observed": obs9,
        "passed": obs9 == exp9,
    })

    # 10. recovery_writes_no_bytes_of_its_own
    exp10 = TORN_TAIL_RECOVERY_SECURITY_MATRIX[9][1]
    obs10 = any(op in ("write", "append") for fs in [fs1, fs2, fs3, fs4] for op, _ in fs._ops)
    checks.append({
        "name": TORN_TAIL_RECOVERY_SECURITY_MATRIX[9][0],
        "expected": exp10,
        "observed": obs10,
        "passed": obs10 == exp10,
    })

    # 11. an_undamaged_log_is_not_truncated_at_all
    exp11 = TORN_TAIL_RECOVERY_SECURITY_MATRIX[10][1]
    fs11 = _LocalRecordingFileSystem({"var/log/app.log": good_buf})
    svc11 = FramedLogService(fs11)
    svc11.open_for_append("var/log/app.log", FsyncPolicy.ALWAYS)
    obs11 = any(op == "truncate" for op, _ in fs11._ops)
    checks.append({
        "name": TORN_TAIL_RECOVERY_SECURITY_MATRIX[10][0],
        "expected": exp11,
        "observed": obs11,
        "passed": obs11 == exp11,
    })

    # 12. an_enormous_declared_length_is_refused_without_a_long_pause
    exp12 = TORN_TAIL_RECOVERY_SECURITY_MATRIX[11][1]
    obs12 = t_elapsed < 1.0
    checks.append({
        "name": TORN_TAIL_RECOVERY_SECURITY_MATRIX[11][0],
        "expected": exp12,
        "observed": obs12,
        "passed": obs12 == exp12,
    })

    # 13. an_empty_log_recovers_to_an_empty_frame_list_without_truncating
    exp13 = TORN_TAIL_RECOVERY_SECURITY_MATRIX[12][1]
    fs13 = _LocalRecordingFileSystem()
    svc13 = FramedLogService(fs13)
    rec13 = svc13.open_for_append("var/log/app.log", FsyncPolicy.ALWAYS)
    obs13 = (len(rec13.frames), rec13.good_end, any(op == "truncate" for op, _ in fs13._ops))
    checks.append({
        "name": TORN_TAIL_RECOVERY_SECURITY_MATRIX[12][0],
        "expected": exp13,
        "observed": obs13,
        "passed": obs13 == exp13,
    })

    # 14. the_recovered_frame_count_is_the_same_for_every_damage_class
    exp14 = TORN_TAIL_RECOVERY_SECURITY_MATRIX[13][1]
    obs14 = (len(rec1.frames), len(rec2.frames), len(rec3.frames), len(rec4.frames))
    checks.append({
        "name": TORN_TAIL_RECOVERY_SECURITY_MATRIX[13][0],
        "expected": exp14,
        "observed": obs14,
        "passed": obs14 == exp14,
    })

    return {
        "case_id": "torn-tail-recovery-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
