# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "errors"
# case = "move_missing_source_raises"
# subject = "shutil.move"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.move: move() of a non-existent source path raises FileNotFoundError (set up in a TemporaryDirectory)"""
import shutil
import tempfile
import os

_raised = False
with tempfile.TemporaryDirectory() as td:
    try:
        shutil.move(os.path.join(td, "nonexistent_src"),
                    os.path.join(td, "dst"))
    except FileNotFoundError:
        _raised = True
assert _raised, "move_missing_source_raises: expected FileNotFoundError"
print("move_missing_source_raises OK")
