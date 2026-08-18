from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
import struct
from typing import Final
import zlib

HEADER_LENGTH: Final[int] = 16
MAX_PAYLOAD_LENGTH: Final[int] = 0xFFFFFFFF

class FrameRejection(str, Enum):
    HEADER_TRUNCATED = "header-truncated"
    PAYLOAD_TRUNCATED = "payload-truncated"
    CHECKSUM_MISMATCH = "checksum-mismatch"

@dataclass(frozen=True)
class LogFrame:
    seq: int
    payload: bytes

def checksum(payload: bytes) -> int:
    return zlib.crc32(payload) & 0xFFFFFFFF

def encode_header(seq: int, payload_length: int, crc: int) -> bytes:
    return struct.pack("<QII", seq, payload_length, crc)

def decode_header(header: bytes) -> tuple[int, int, int]:
    if len(header) != HEADER_LENGTH:
        raise ValueError(f"Header length must be {HEADER_LENGTH}, got {len(header)}")
    seq: int
    payload_length: int
    crc: int
    seq, payload_length, crc = struct.unpack("<QII", header)
    return (seq, payload_length, crc)

def encode_frame(seq: int, payload: bytes) -> bytes:
    crc = checksum(payload)
    hdr = encode_header(seq, len(payload), crc)
    return hdr + payload

def read_frame_at(buffer: bytes, offset: int) -> tuple[LogFrame, int] | FrameRejection:
    remaining = len(buffer) - offset
    if remaining < HEADER_LENGTH:
        return FrameRejection.HEADER_TRUNCATED
    seq, declared_length, stored_crc = decode_header(buffer[offset : offset + HEADER_LENGTH])
    body_start = offset + HEADER_LENGTH
    available = len(buffer) - body_start
    if declared_length > available:
        return FrameRejection.PAYLOAD_TRUNCATED
    payload = buffer[body_start : body_start + declared_length]
    if checksum(payload) != stored_crc:
        return FrameRejection.CHECKSUM_MISMATCH
    return (LogFrame(seq, payload), body_start + declared_length)
