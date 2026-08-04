from __future__ import annotations

from dataclasses import dataclass
from typing import Sequence

from storage_durable.domain.frame import (
    FrameRejection,
    LogFrame,
    read_frame_at,
)

@dataclass(frozen=True)
class ScanResult:
    frames: tuple[LogFrame, ...]
    good_end: int
    rejection: FrameRejection | None

def scan(buffer: bytes) -> ScanResult:
    frames: list[LogFrame] = []
    offset = 0
    while True:
        if offset == len(buffer):
            return ScanResult(tuple(frames), offset, None)
        outcome = read_frame_at(buffer, offset)
        if isinstance(outcome, FrameRejection):
            return ScanResult(tuple(frames), offset, outcome)
        frame, next_offset = outcome
        frames.append(frame)
        offset = next_offset

def frames_after(frames: Sequence[LogFrame], from_seq: int) -> tuple[LogFrame, ...]:
    return tuple(f for f in frames if f.seq > from_seq)

def highest_seq(frames: Sequence[LogFrame]) -> int:
    if not frames:
        return 0
    return max(f.seq for f in frames)
