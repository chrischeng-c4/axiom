# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "rmtree_removes_nested"
# subject = "shutil.rmtree"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.rmtree: rmtree removes a deeply nested directory tree (a/b/c with a file) so the top-level directory no longer exists"""
import shutil
import tempfile
import os

with tempfile.TemporaryDirectory() as d:
    sub = os.path.join(d, "a", "b", "c")
    os.makedirs(sub)
    with open(os.path.join(sub, "file.txt"), "w") as f:
        f.write("x")
    root = os.path.join(d, "a")
    shutil.rmtree(root)
    assert not os.path.exists(root), "nested rmtree"

print("rmtree_removes_nested OK")
