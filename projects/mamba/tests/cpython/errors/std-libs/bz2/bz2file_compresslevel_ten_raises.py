# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "errors"
# case = "bz2file_compresslevel_ten_raises"
# subject = "bz2.BZ2File"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2File: bz2file_compresslevel_ten_raises (errors)."""
import bz2, io

_raised = False
try:
    bz2.BZ2File(io.BytesIO(), "w", compresslevel=10)
except ValueError:
    _raised = True
assert _raised, "bz2file_compresslevel_ten_raises: expected ValueError"
print("bz2file_compresslevel_ten_raises OK")
