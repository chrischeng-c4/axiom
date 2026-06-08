# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "errors"
# case = "bz2file_float_filename_raises"
# subject = "bz2.BZ2File"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2File: bz2file_float_filename_raises (errors)."""
import bz2

_raised = False
try:
    bz2.BZ2File(123.456)
except TypeError:
    _raised = True
assert _raised, "bz2file_float_filename_raises: expected TypeError"
print("bz2file_float_filename_raises OK")
