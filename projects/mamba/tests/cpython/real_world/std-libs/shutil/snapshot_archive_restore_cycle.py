# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "real_world"
# case = "snapshot_archive_restore_cycle"
# subject = "shutil"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil: a backup tool stages a project tree, copytree-snapshots it, make_archive packs the snapshot to a zip, unpack_archive restores it into a fresh dir, and the restored content matches the original; rmtree cleans up — all inside one TemporaryDirectory"""
import shutil
import tempfile
import os

with tempfile.TemporaryDirectory() as td:
    # 1. Stage a small "project" tree.
    project = os.path.join(td, "project")
    os.makedirs(os.path.join(project, "src"))
    os.makedirs(os.path.join(project, "docs"))
    with open(os.path.join(project, "README.md"), "w", encoding="utf-8") as f:
        f.write("# Project\n")
    with open(os.path.join(project, "src", "main.py"), "w", encoding="utf-8") as f:
        f.write("print('hello')\n")
    with open(os.path.join(project, "docs", "guide.txt"), "w", encoding="utf-8") as f:
        f.write("user guide\n")

    # 2. Snapshot the tree with copytree.
    snapshot = os.path.join(td, "snapshot")
    shutil.copytree(project, snapshot)
    assert os.path.isfile(os.path.join(snapshot, "src", "main.py")), "snapshot built"

    # 3. Pack the snapshot into a zip archive.
    archive_base = os.path.join(td, "backup")
    archive = shutil.make_archive(archive_base, "zip", root_dir=snapshot)
    assert os.path.isfile(archive), f"archive created: {archive!r}"
    assert archive.endswith(".zip"), f"archive ext = {archive!r}"

    # 4. Restore into a fresh directory.
    restored = os.path.join(td, "restored")
    shutil.unpack_archive(archive, restored, "zip")

    # 5. Restored content must match the original.
    with open(os.path.join(restored, "README.md"), encoding="utf-8") as f:
        assert f.read() == "# Project\n", "README restored"
    with open(os.path.join(restored, "src", "main.py"), encoding="utf-8") as f:
        assert f.read() == "print('hello')\n", "main.py restored"
    with open(os.path.join(restored, "docs", "guide.txt"), encoding="utf-8") as f:
        assert f.read() == "user guide\n", "guide.txt restored"

    # 6. Clean up the working snapshot with rmtree.
    shutil.rmtree(snapshot)
    assert not os.path.exists(snapshot), "snapshot cleaned up"

print("snapshot_archive_restore_cycle OK")
