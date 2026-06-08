# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "errors"
# case = "copytree_existing_dest_raises"
# subject = "shutil.copytree"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.copytree: copytree() onto an already-existing destination directory raises FileExistsError (both dirs in a TemporaryDirectory)"""
import shutil
import tempfile
import os

_raised = False
with tempfile.TemporaryDirectory() as td:
    a = os.path.join(td, "a")
    b = os.path.join(td, "b")
    os.mkdir(a)
    os.mkdir(b)
    try:
        shutil.copytree(a, b)
    except FileExistsError:
        _raised = True
assert _raised, "copytree_existing_dest_raises: expected FileExistsError"
print("copytree_existing_dest_raises OK")
