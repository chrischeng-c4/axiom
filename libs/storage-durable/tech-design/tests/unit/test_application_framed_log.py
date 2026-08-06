from __future__ import annotations

from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from storage_durable.application.framed_log import FramedLogService
from storage_durable.domain.frame import (
    HEADER_LENGTH,
    FrameRejection,
    LogFrame,
    encode_frame,
)
from storage_durable.domain.fsync_policy import FsyncPolicy
from storage_durable.infrastructure.memory_filesystem import MemoryFileSystem

class TestApplicationFramedLog(unittest.TestCase):
    def test_open_for_append_recovery_with_torn_tail(self) -> None:
        p1, p2, p3 = b"first", b"second", b"third"
        f1 = encode_frame(1, p1)
        f2 = encode_frame(2, p2)
        f3 = encode_frame(3, p3)
        good_end = (HEADER_LENGTH + len(p1)) + (HEADER_LENGTH + len(p2)) + (HEADER_LENGTH + len(p3))
        self.assertEqual(good_end, len(f1) + len(f2) + len(f3))

        damaged_log = f1 + f2 + f3 + b"garbage99"  # 9 bytes garbage
        fs = MemoryFileSystem({"var/log/app.log": damaged_log})
        svc = FramedLogService(fs)

        recovered = svc.open_for_append("var/log/app.log", FsyncPolicy.ALWAYS)
        self.assertEqual(recovered.good_end, good_end)
        self.assertEqual(recovered.original_length, len(damaged_log))
        self.assertTrue(recovered.truncated)
        self.assertEqual(recovered.rejection, FrameRejection.HEADER_TRUNCATED)
        self.assertEqual(len(recovered.frames), 3)

        ops = [(op.name, op.path) for op in fs.operations()]
        self.assertIn(("truncate", "var/log/app.log"), ops)
        self.assertEqual(len(fs.read("var/log/app.log") or b""), good_end)

    def test_open_for_append_undamaged_log_no_truncate(self) -> None:
        f1 = encode_frame(1, b"hello")
        fs = MemoryFileSystem({"var/log/app.log": f1})
        svc = FramedLogService(fs)

        recovered = svc.open_for_append("var/log/app.log", FsyncPolicy.ALWAYS)
        self.assertEqual(recovered.good_end, len(f1))
        self.assertFalse(recovered.truncated)
        self.assertIsNone(recovered.rejection)

        ops = [op.name for op in fs.operations()]
        self.assertNotIn("truncate", ops)

    def test_append_success_and_sync_policy(self) -> None:
        fs = MemoryFileSystem()
        svc = FramedLogService(fs)

        svc.append("var/log/app.log", 1, b"payload1", FsyncPolicy.ALWAYS)
        ops1 = [op.name for op in fs.operations()]
        self.assertIn("sync_file", ops1)

        fs_os = MemoryFileSystem()
        svc_os = FramedLogService(fs_os)
        svc_os.append("var/log/app.log", 2, b"payload2", FsyncPolicy.OS)
        ops2 = [op.name for op in fs_os.operations()]
        self.assertNotIn("sync_file", ops2)

    def test_append_exceeds_max_payload_length(self) -> None:
        fs = MemoryFileSystem()
        svc = FramedLogService(fs)

        class DummyHugeBytes(bytes):
            def __len__(self) -> int:
                return 0xFFFFFFFF + 1

        huge_payload = DummyHugeBytes()
        with self.assertRaises(ValueError):
            svc.append("var/log/app.log", 1, huge_payload, FsyncPolicy.ALWAYS)

    def test_replay_filters_strictly_after_from_seq(self) -> None:
        f1 = encode_frame(1, b"one")
        f5 = encode_frame(5, b"five")
        f10 = encode_frame(10, b"ten")
        fs = MemoryFileSystem({"var/log/app.log": f1 + f5 + f10})
        svc = FramedLogService(fs)

        frames, highest = svc.replay("var/log/app.log", from_seq=5)
        self.assertEqual(len(frames), 1)
        self.assertEqual(frames[0], LogFrame(10, b"ten"))
        self.assertEqual(highest, 10)

    def test_compact_through_retains_strictly_after_seq(self) -> None:
        f1 = encode_frame(1, b"one")
        f5 = encode_frame(5, b"five")
        f10 = encode_frame(10, b"ten")
        fs = MemoryFileSystem({"var/log/app.log": f1 + f5 + f10})
        svc = FramedLogService(fs)

        retained_count = svc.compact_through("var/log/app.log", through_seq=5, policy=FsyncPolicy.ALWAYS)
        self.assertEqual(retained_count, 1)

        # verify compacted log on disk only has frame 10
        compacted_buf = fs.read("var/log/app.log") or b""
        self.assertEqual(compacted_buf, f10)

    def test_open_for_append_empty_log(self) -> None:
        fs = MemoryFileSystem()
        svc = FramedLogService(fs)

        recovered = svc.open_for_append("var/log/app.log", FsyncPolicy.ALWAYS)
        self.assertEqual(recovered.good_end, 0)
        self.assertEqual(recovered.original_length, 0)
        self.assertEqual(recovered.frames, ())
        self.assertIsNone(recovered.rejection)

if __name__ == "__main__":
    unittest.main()
