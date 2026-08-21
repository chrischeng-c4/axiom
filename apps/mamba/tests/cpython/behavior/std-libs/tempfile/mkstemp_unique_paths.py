# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "mkstemp_unique_paths"
# subject = "tempfile.mkstemp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.mkstemp: two mkstemp() calls return two distinct paths"""
import os
import tempfile

_fd1, _p1 = tempfile.mkstemp()
_fd2, _p2 = tempfile.mkstemp()
try:
    assert _p1 != _p2, f"mkstemp unique: {_p1} vs {_p2}"
finally:
    os.close(_fd1); os.unlink(_p1)
    os.close(_fd2); os.unlink(_p2)
print("mkstemp_unique_paths OK")
