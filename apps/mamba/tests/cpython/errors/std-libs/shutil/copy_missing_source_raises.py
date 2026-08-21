# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "errors"
# case = "copy_missing_source_raises"
# subject = "shutil.copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.copy: copy() of a non-existent source path raises FileNotFoundError (set up in a TemporaryDirectory)"""
import shutil
import tempfile
import os

_raised = False
with tempfile.TemporaryDirectory() as td:
    try:
        shutil.copy(os.path.join(td, "nonexistent"), os.path.join(td, "dst"))
    except FileNotFoundError:
        _raised = True
assert _raised, "copy_missing_source_raises: expected FileNotFoundError"
print("copy_missing_source_raises OK")
