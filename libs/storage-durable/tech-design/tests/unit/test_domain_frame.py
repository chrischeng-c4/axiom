from __future__ import annotations

from pathlib import Path
import struct
import sys
import unittest
import zlib

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from storage_durable.domain.frame import (
    HEADER_LENGTH,
    FrameRejection,
    LogFrame,
    checksum,
    decode_header,
    encode_frame,
    encode_header,
    read_frame_at,
)

class TestDomainFrame(unittest.TestCase):
    def test_decode_header_explicit_struct_pack(self) -> None:
        crc_val = zlib.crc32(b"hello") & 0xFFFFFFFF
        self.assertEqual(crc_val, 907060870)
        raw_hdr = struct.pack("<QII", 7, 5, crc_val)
        decoded = decode_header(raw_hdr)
        self.assertEqual(decoded, (7, 5, 907060870))

    def test_decode_header_invalid_length(self) -> None:
        short_hdr = b"\x00" * 15
        with self.assertRaises(ValueError):
            decode_header(short_hdr)

    def test_encode_and_read_frame_roundtrip(self) -> None:
        payload = b"hello"
        frame_bytes = encode_frame(7, payload)
        self.assertEqual(len(frame_bytes), 21)
        expected_hdr = encode_header(7, 5, 907060870)
        self.assertEqual(frame_bytes[:16], expected_hdr)
        self.assertEqual(frame_bytes[16:], b"hello")
        res = read_frame_at(frame_bytes, 0)
        self.assertIsInstance(res, tuple)
        frame, next_offset = res  # type: ignore[misc]
        self.assertEqual(frame, LogFrame(7, b"hello"))
        self.assertEqual(next_offset, 21)

    def test_read_frame_header_truncated(self) -> None:
        buf = b"\x00" * 10
        res = read_frame_at(buf, 0)
        self.assertEqual(res, FrameRejection.HEADER_TRUNCATED)

    def test_read_frame_payload_truncated_one_byte_past_end(self) -> None:
        hdr = encode_header(1, 5, checksum(b"hello"))
        buf = hdr + b"hell"  # 4 bytes payload instead of 5
        res = read_frame_at(buf, 0)
        self.assertEqual(res, FrameRejection.PAYLOAD_TRUNCATED)

    def test_read_frame_payload_truncated_enormous_declared_length(self) -> None:
        hdr = encode_header(7, 0xFFFFFFFF, 12345)
        buf = hdr + b"1234"
        res = read_frame_at(buf, 0)
        self.assertEqual(res, FrameRejection.PAYLOAD_TRUNCATED)

    def test_read_frame_checksum_mismatch(self) -> None:
        hdr = encode_header(1, 5, checksum(b"hello"))
        buf = hdr + b"hallo"
        res = read_frame_at(buf, 0)
        self.assertEqual(res, FrameRejection.CHECKSUM_MISMATCH)

if __name__ == "__main__":
    unittest.main()
