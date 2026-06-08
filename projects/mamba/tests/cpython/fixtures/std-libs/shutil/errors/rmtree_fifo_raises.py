# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "errors"
# case = "rmtree_fifo_raises"
# subject = "shutil.rmtree"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.rmtree: rmtree() on a named pipe (FIFO) raises NotADirectoryError on POSIX; guarded by hasattr(os, 'mkfifo') so non-POSIX platforms exit 0 via a skip path"""
import shutil
import tempfile
import os

# POSIX-only path: rmtree on a FIFO refuses (it is not a directory). On a
# platform without os.mkfifo the case is structurally skipped but still exits 0.
if hasattr(os, "mkfifo"):
    _raised = False
    with tempfile.TemporaryDirectory() as td:
        fifo = os.path.join(td, "mypipe")
        os.mkfifo(fifo)
        try:
            shutil.rmtree(fifo)
        except NotADirectoryError:
            _raised = True
    assert _raised, "rmtree_fifo_raises: expected NotADirectoryError"

print("rmtree_fifo_raises OK")
