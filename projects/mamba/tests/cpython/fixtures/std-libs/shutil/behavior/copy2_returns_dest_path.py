# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "copy2_returns_dest_path"
# subject = "shutil.copy2"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.copy2: copy2() returns the destination path string and the destination file exists after the copy"""
import shutil
import tempfile
import os

with tempfile.TemporaryDirectory() as d:
    src = os.path.join(d, "s.txt")
    dst = os.path.join(d, "d.txt")
    with open(src, "w") as f:
        f.write("copy2 test")
    result = shutil.copy2(src, dst)
    assert isinstance(result, str), f"copy2 returns path = {type(result)!r}"
    assert os.path.exists(dst), "copy2 dst exists"

print("copy2_returns_dest_path OK")
