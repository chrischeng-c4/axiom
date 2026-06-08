# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "copytree_copies_tree"
# subject = "shutil.copytree"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.copytree: copytree recreates a directory tree (a top-level file plus a subdir file) at the destination with identical content"""
import shutil
import tempfile
import os

with tempfile.TemporaryDirectory() as d:
    src_dir = os.path.join(d, "src_tree")
    dst_dir = os.path.join(d, "dst_tree")
    os.makedirs(os.path.join(src_dir, "sub"))
    with open(os.path.join(src_dir, "a.txt"), "w") as f:
        f.write("a")
    with open(os.path.join(src_dir, "sub", "b.txt"), "w") as f:
        f.write("b")
    shutil.copytree(src_dir, dst_dir)
    assert os.path.isdir(os.path.join(dst_dir, "sub")), "sub dir copied"
    with open(os.path.join(dst_dir, "a.txt")) as f2:
        assert f2.read() == "a", "a.txt copied"
    with open(os.path.join(dst_dir, "sub", "b.txt")) as f3:
        assert f3.read() == "b", "b.txt copied"

print("copytree_copies_tree OK")
