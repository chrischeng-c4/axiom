# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "move_renames_file"
# subject = "shutil.move"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.move: move() relocates a file: the source path disappears, the destination appears, and the content is preserved"""
import shutil
import tempfile
import os

with tempfile.TemporaryDirectory() as d:
    src = os.path.join(d, "old.txt")
    dst = os.path.join(d, "new.txt")
    with open(src, "w") as f:
        f.write("moving")
    shutil.move(src, dst)
    assert not os.path.exists(src), "source gone after move"
    assert os.path.exists(dst), "dest exists after move"
    with open(dst) as f2:
        assert f2.read() == "moving", "content preserved"

print("move_renames_file OK")
