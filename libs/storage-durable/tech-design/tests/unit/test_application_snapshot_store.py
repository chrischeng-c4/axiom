from __future__ import annotations

from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from storage_durable.application.durable_replace import DurableReplaceService
from storage_durable.application.snapshot_store import (
    SnapshotEntry,
    SnapshotStoreConfig,
    SnapshotStoreService,
)
from storage_durable.domain.fsync_policy import FsyncPolicy
from storage_durable.infrastructure.memory_filesystem import MemoryFileSystem

class TestApplicationSnapshotStore(unittest.TestCase):
    def test_save_and_entries_ordering(self) -> None:
        cfg = SnapshotStoreConfig(root="var/snaps", prefix="snap", extension="bin", policy=FsyncPolicy.ALWAYS)

        # Order 1: save 9 then 10
        fs1 = MemoryFileSystem()
        replace1 = DurableReplaceService(fs1)
        svc1 = SnapshotStoreService(fs1, replace1)
        svc1.save(cfg, 9, b"snap9")
        svc1.save(cfg, 10, b"snap10")
        entries1 = svc1.entries(cfg)
        self.assertEqual(entries1, (SnapshotEntry(9, "snap-9.bin"), SnapshotEntry(10, "snap-10.bin")))

        # Order 2: save 10 then 9
        fs2 = MemoryFileSystem()
        replace2 = DurableReplaceService(fs2)
        svc2 = SnapshotStoreService(fs2, replace2)
        svc2.save(cfg, 10, b"snap10")
        svc2.save(cfg, 9, b"snap9")
        entries2 = svc2.entries(cfg)
        self.assertEqual(entries2, (SnapshotEntry(9, "snap-9.bin"), SnapshotEntry(10, "snap-10.bin")))

    def test_load_latest_returns_highest_sequence(self) -> None:
        cfg = SnapshotStoreConfig(root="var/snaps", prefix="snap", extension="bin", policy=FsyncPolicy.ALWAYS)
        fs = MemoryFileSystem()
        replace = DurableReplaceService(fs)
        svc = SnapshotStoreService(fs, replace)

        svc.save(cfg, 10, b"data-10")
        svc.save(cfg, 9, b"data-9")

        latest = svc.load_latest(cfg)
        self.assertEqual(latest, b"data-10")

    def test_prune_below_equal_above_keep(self) -> None:
        cfg = SnapshotStoreConfig(root="var/snaps", prefix="snap", extension="bin", policy=FsyncPolicy.ALWAYS)
        fs = MemoryFileSystem()
        replace = DurableReplaceService(fs)
        svc = SnapshotStoreService(fs, replace)

        svc.save(cfg, 1, b"s1")
        svc.save(cfg, 2, b"s2")
        svc.save(cfg, 3, b"s3")

        # keep >= len
        self.assertEqual(svc.prune(cfg, keep=5), 0)
        self.assertEqual(svc.prune(cfg, keep=3), 0)

        # keep = 1: removes lowest 2 (seq 1 and seq 2)
        removed = svc.prune(cfg, keep=1)
        self.assertEqual(removed, 2)

        remaining = svc.entries(cfg)
        self.assertEqual(remaining, (SnapshotEntry(3, "snap-3.bin"),))

    def test_prune_preserves_foreign_files(self) -> None:
        cfg = SnapshotStoreConfig(root="var/snaps", prefix="snap", extension="bin", policy=FsyncPolicy.ALWAYS)
        fs = MemoryFileSystem({"var/snaps/foreign.txt": b"keep me"})
        replace = DurableReplaceService(fs)
        svc = SnapshotStoreService(fs, replace)

        svc.save(cfg, 1, b"s1")
        svc.save(cfg, 2, b"s2")

        removed = svc.prune(cfg, keep=1)
        self.assertEqual(removed, 1)
        self.assertTrue(fs.exists("var/snaps/foreign.txt"))
        self.assertFalse(fs.exists("var/snaps/snap-1.bin"))
        self.assertTrue(fs.exists("var/snaps/snap-2.bin"))

    def test_prune_negative_keep_raises_value_error(self) -> None:
        cfg = SnapshotStoreConfig(root="var/snaps", prefix="snap", extension="bin", policy=FsyncPolicy.ALWAYS)
        fs = MemoryFileSystem()
        replace = DurableReplaceService(fs)
        svc = SnapshotStoreService(fs, replace)

        with self.assertRaises(ValueError):
            svc.prune(cfg, keep=-1)

    def test_load_latest_empty_returns_none(self) -> None:
        cfg = SnapshotStoreConfig(root="var/snaps", prefix="snap", extension="bin", policy=FsyncPolicy.ALWAYS)
        fs = MemoryFileSystem()
        replace = DurableReplaceService(fs)
        svc = SnapshotStoreService(fs, replace)

        self.assertIsNone(svc.load_latest(cfg))

if __name__ == "__main__":
    unittest.main()
