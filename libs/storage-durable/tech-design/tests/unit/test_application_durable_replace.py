from __future__ import annotations

from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from storage_durable.application.durable_replace import (
    DurableReplaceService,
    ReplaceRequest,
)
from storage_durable.domain.failure import FailureKind, StorageFailure
from storage_durable.domain.fsync_policy import FsyncPolicy
from storage_durable.infrastructure.memory_filesystem import MemoryFileSystem

class TestApplicationDurableReplace(unittest.TestCase):
    def test_replace_operation_sequence_forcing_policy(self) -> None:
        for policy in [FsyncPolicy.ALWAYS, FsyncPolicy.EVERY_SEC, FsyncPolicy.INTERVAL]:
            with self.subTest(policy=policy):
                fs = MemoryFileSystem()
                svc = DurableReplaceService(fs)
                svc.replace(ReplaceRequest("var/state/x.bin", b"data", policy))
                ops = [(op.name, op.path) for op in fs.operations()]
                expected = [
                    ("make_directories", "var/state"),
                    ("remove", "var/state/x.bin.tmp"),
                    ("write", "var/state/x.bin.tmp"),
                    ("sync_file", "var/state/x.bin.tmp"),
                    ("rename", "var/state/x.bin.tmp"),
                    ("sync_directory", "var/state"),
                ]
                self.assertEqual(ops, expected)
                self.assertEqual(fs.read("var/state/x.bin"), b"data")

    def test_replace_operation_sequence_os_policy(self) -> None:
        fs = MemoryFileSystem()
        svc = DurableReplaceService(fs)
        svc.replace(ReplaceRequest("var/state/x.bin", b"data", FsyncPolicy.OS))
        ops = [(op.name, op.path) for op in fs.operations()]
        expected = [
            ("make_directories", "var/state"),
            ("remove", "var/state/x.bin.tmp"),
            ("write", "var/state/x.bin.tmp"),
            ("rename", "var/state/x.bin.tmp"),
        ]
        self.assertEqual(ops, expected)
        self.assertEqual(fs.read("var/state/x.bin"), b"data")

    def test_replace_write_failure_leaves_target_absent(self) -> None:
        fs = MemoryFileSystem()
        fs.fail_once("write", "var/state/x.bin.tmp", FailureKind.IO)
        svc = DurableReplaceService(fs)
        with self.assertRaises(StorageFailure):
            svc.replace(ReplaceRequest("var/state/x.bin", b"data", FsyncPolicy.ALWAYS))
        self.assertFalse(fs.exists("var/state/x.bin"))

    def test_replace_cleans_stale_staging_file(self) -> None:
        fs = MemoryFileSystem({"var/state/x.bin.tmp": b"stale garbage"})
        svc = DurableReplaceService(fs)
        svc.replace(ReplaceRequest("var/state/x.bin", b"fresh data", FsyncPolicy.ALWAYS))
        self.assertEqual(fs.read("var/state/x.bin"), b"fresh data")
        self.assertFalse(fs.exists("var/state/x.bin.tmp"))

    def test_replace_swallows_tolerated_directory_sync_failure(self) -> None:
        for kind in [FailureKind.PERMISSION_DENIED, FailureKind.UNSUPPORTED, FailureKind.OTHER]:
            with self.subTest(kind=kind):
                fs = MemoryFileSystem()
                fs.fail_once("sync_directory", "var/state", kind)
                svc = DurableReplaceService(fs)
                svc.replace(ReplaceRequest("var/state/x.bin", b"data", FsyncPolicy.ALWAYS))
                self.assertEqual(fs.read("var/state/x.bin"), b"data")

    def test_replace_raises_untolerated_directory_sync_failure(self) -> None:
        for kind in [FailureKind.IO, FailureKind.NOT_FOUND]:
            with self.subTest(kind=kind):
                fs = MemoryFileSystem()
                fs.fail_once("sync_directory", "var/state", kind)
                svc = DurableReplaceService(fs)
                with self.assertRaises(StorageFailure) as ctx:
                    svc.replace(ReplaceRequest("var/state/x.bin", b"data", FsyncPolicy.ALWAYS))
                self.assertEqual(ctx.exception.kind, kind)

if __name__ == "__main__":
    unittest.main()
