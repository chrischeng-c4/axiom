# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "real_world"
# case = "directory_tree_walk_and_size"
# subject = "os.walk"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.walk: a backup-sizing tool builds a small directory tree under a TemporaryDirectory, os.walk recurses it bottom-and-top, and the per-file os.path.getsize total matches the bytes written"""
import os
import tempfile

# A backup-sizing tool: lay down a small nested project tree, then sum the
# on-disk byte total two ways (os.walk top-down and bottom-up) and confirm both
# agree with the bytes we actually wrote.
layout = {
    os.path.join("src", "main.py"): b"print('hello')\n",
    os.path.join("src", "util", "io.py"): b"# io helpers\n",
    os.path.join("docs", "README.md"): b"# Project\n\nNotes.\n",
    "config.ini": b"[core]\nname=demo\n",
}
expected_total = sum(len(v) for v in layout.values())

with tempfile.TemporaryDirectory() as root:
    # Build the tree.
    for rel, payload in layout.items():
        full = os.path.join(root, rel)
        os.makedirs(os.path.dirname(full), exist_ok=True) if os.path.dirname(rel) else None
        with open(full, "wb") as f:
            f.write(payload)

    # Walk top-down: aggregate file count + byte total via os.path.getsize.
    seen_files = 0
    total_topdown = 0
    dir_names = set()
    for dirpath, dirnames, filenames in os.walk(root, topdown=True):
        for name in dirnames:
            dir_names.add(name)
        for name in filenames:
            fp = os.path.join(dirpath, name)
            assert os.path.isfile(fp), f"walked entry not a file: {fp!r}"
            total_topdown += os.path.getsize(fp)
            seen_files += 1

    # Walk bottom-up: must visit exactly the same files / bytes.
    total_bottomup = 0
    files_bottomup = 0
    for dirpath, dirnames, filenames in os.walk(root, topdown=False):
        for name in filenames:
            total_bottomup += os.path.getsize(os.path.join(dirpath, name))
            files_bottomup += 1

    assert seen_files == len(layout), f"file count = {seen_files}"
    assert files_bottomup == len(layout), f"bottom-up file count = {files_bottomup}"
    assert total_topdown == expected_total, f"top-down bytes = {total_topdown}"
    assert total_bottomup == expected_total, "bottom-up bytes match top-down"
    assert {"src", "util", "docs"} <= dir_names, f"dirs walked = {dir_names!r}"
print("directory_tree_walk_and_size OK")
