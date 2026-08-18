from __future__ import annotations

import struct
from typing import Mapping
import zlib

from storage_durable.application.framed_log import FramedLogService
from storage_durable.domain.failure import FailureKind, StorageFailure
from storage_durable.domain.frame import (
    HEADER_LENGTH,
    decode_header,
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

TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX = (
    ("the_design_decodes_a_header_the_contract_packed_itself", (42, 5, 907060870)),
    ("the_design_encodes_a_header_the_contract_can_unpack_itself", (15, 4, 9999)),
    ("an_encoded_frame_is_its_header_followed_by_its_payload_bytes", True),
    ("the_header_width_is_the_declared_constant", 16),
    ("a_round_trip_through_the_scan_preserves_every_sequence", (10, 20, 30)),
    ("a_round_trip_through_the_scan_preserves_every_payload", ((112, 49, 48), (112, 50, 48), (112, 51, 48))),
    ("replay_returns_only_sequences_strictly_greater_than_the_requested_one", (10,)),
    ("replay_preserves_the_order_the_frames_were_written_in", (1, 5, 10)),
    ("replay_reports_the_highest_sequence_among_the_frames_it_returned", 10),
    ("replay_from_below_the_first_sequence_returns_the_whole_log", (5, 10)),
    ("replay_past_the_last_sequence_returns_no_frames", 0),
    ("replay_past_the_last_sequence_reports_a_high_water_of_zero", 0),
    ("compaction_retains_exactly_the_frames_past_the_retention_point", (10,)),
    ("compaction_reports_the_number_of_frames_it_retained", 1),
)

def verify_torn_tail_recovery_behavior() -> dict[str, object]:
    checks = []

    # 1. the_design_decodes_a_header_the_contract_packed_itself
    exp1 = TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[0][1]
    crc1 = zlib.crc32(b"hello") & 0xFFFFFFFF
    hdr1 = struct.pack("<QII", 42, 5, crc1)
    obs1 = decode_header(hdr1)
    checks.append({
        "name": TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[0][0],
        "expected": exp1,
        "observed": obs1,
        "passed": obs1 == exp1,
    })

    # 2. the_design_encodes_a_header_the_contract_can_unpack_itself
    exp2 = TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[1][1]
    enc_hdr2 = encode_header(15, 4, 9999)
    obs2 = struct.unpack("<QII", enc_hdr2)
    checks.append({
        "name": TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[1][0],
        "expected": exp2,
        "observed": obs2,
        "passed": obs2 == exp2,
    })

    # 3. an_encoded_frame_is_its_header_followed_by_its_payload_bytes
    exp3 = TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[2][1]
    frame3 = encode_frame(1, b"test")
    hdr3 = encode_header(1, 4, zlib.crc32(b"test") & 0xFFFFFFFF)
    obs3 = (frame3[:16] == hdr3) and (frame3[16:] == b"test")
    checks.append({
        "name": TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[2][0],
        "expected": exp3,
        "observed": obs3,
        "passed": obs3 == exp3,
    })

    # 4. the_header_width_is_the_declared_constant
    exp4 = TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[3][1]
    obs4 = HEADER_LENGTH
    checks.append({
        "name": TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[3][0],
        "expected": exp4,
        "observed": obs4,
        "passed": obs4 == exp4,
    })

    # 5. a_round_trip_through_the_scan_preserves_every_sequence
    exp5 = TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[4][1]
    f10 = encode_frame(10, b"p10")
    f20 = encode_frame(20, b"p20")
    f30 = encode_frame(30, b"p30")
    buf5 = f10 + f20 + f30
    fs5 = _LocalRecordingFileSystem({"var/log/app.log": buf5})
    svc5 = FramedLogService(fs5)
    rec5 = svc5.open_for_append("var/log/app.log", FsyncPolicy.ALWAYS)
    obs5 = tuple(f.seq for f in rec5.frames)
    checks.append({
        "name": TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[4][0],
        "expected": exp5,
        "observed": obs5,
        "passed": obs5 == exp5,
    })

    # 6. a_round_trip_through_the_scan_preserves_every_payload
    exp6 = TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[5][1]
    obs6 = tuple(tuple(f.payload) for f in rec5.frames)
    checks.append({
        "name": TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[5][0],
        "expected": exp6,
        "observed": obs6,
        "passed": obs6 == exp6,
    })

    # 7. replay_returns_only_sequences_strictly_greater_than_the_requested_one
    exp7 = TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[6][1]
    g1 = encode_frame(1, b"one")
    g5 = encode_frame(5, b"five")
    g10 = encode_frame(10, b"ten")
    fs7 = _LocalRecordingFileSystem({"var/log/app.log": g1 + g5 + g10})
    svc7 = FramedLogService(fs7)
    frames7, _ = svc7.replay("var/log/app.log", from_seq=5)
    obs7 = tuple(f.seq for f in frames7)
    checks.append({
        "name": TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[6][0],
        "expected": exp7,
        "observed": obs7,
        "passed": obs7 == exp7,
    })

    # 8. replay_preserves_the_order_the_frames_were_written_in
    exp8 = TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[7][1]
    frames8, _ = svc7.replay("var/log/app.log", from_seq=0)
    obs8 = tuple(f.seq for f in frames8)
    checks.append({
        "name": TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[7][0],
        "expected": exp8,
        "observed": obs8,
        "passed": obs8 == exp8,
    })

    # 9. replay_reports_the_highest_sequence_among_the_frames_it_returned
    exp9 = TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[8][1]
    _, high9 = svc7.replay("var/log/app.log", from_seq=5)
    obs9 = high9
    checks.append({
        "name": TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[8][0],
        "expected": exp9,
        "observed": obs9,
        "passed": obs9 == exp9,
    })

    # 10. replay_from_below_the_first_sequence_returns_the_whole_log
    exp10 = TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[9][1]
    fs10 = _LocalRecordingFileSystem({"var/log/app.log": g5 + g10})
    svc10 = FramedLogService(fs10)
    frames10, _ = svc10.replay("var/log/app.log", from_seq=0)
    obs10 = tuple(f.seq for f in frames10)
    checks.append({
        "name": TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[9][0],
        "expected": exp10,
        "observed": obs10,
        "passed": obs10 == exp10,
    })

    # 11. replay_past_the_last_sequence_returns_no_frames
    exp11 = TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[10][1]
    frames11, _ = svc7.replay("var/log/app.log", from_seq=15)
    obs11 = len(frames11)
    checks.append({
        "name": TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[10][0],
        "expected": exp11,
        "observed": obs11,
        "passed": obs11 == exp11,
    })

    # 12. replay_past_the_last_sequence_reports_a_high_water_of_zero
    exp12 = TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[11][1]
    _, high12 = svc7.replay("var/log/app.log", from_seq=15)
    obs12 = high12
    checks.append({
        "name": TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[11][0],
        "expected": exp12,
        "observed": obs12,
        "passed": obs12 == exp12,
    })

    # 13. compaction_retains_exactly_the_frames_past_the_retention_point
    exp13 = TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[12][1]
    svc7.compact_through("var/log/app.log", through_seq=5, policy=FsyncPolicy.ALWAYS)
    frames13, _ = svc7.replay("var/log/app.log", from_seq=0)
    obs13 = tuple(f.seq for f in frames13)
    checks.append({
        "name": TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[12][0],
        "expected": exp13,
        "observed": obs13,
        "passed": obs13 == exp13,
    })

    # 14. compaction_reports_the_number_of_frames_it_retained
    exp14 = TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[13][1]
    fs14 = _LocalRecordingFileSystem({"var/log/app.log": g1 + g5 + g10})
    svc14 = FramedLogService(fs14)
    retained_count14 = svc14.compact_through("var/log/app.log", through_seq=5, policy=FsyncPolicy.ALWAYS)
    obs14 = retained_count14
    checks.append({
        "name": TORN_TAIL_RECOVERY_BEHAVIOR_MATRIX[13][0],
        "expected": exp14,
        "observed": obs14,
        "passed": obs14 == exp14,
    })

    return {
        "case_id": "torn-tail-recovery-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
