# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "errors"
# case = "compress_level_zero_raises"
# subject = "bz2.compress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.compress: compress_level_zero_raises (errors)."""
import bz2

_raised = False
try:
    bz2.compress(b"x", 0)
except ValueError:
    _raised = True
assert _raised, "compress_level_zero_raises: expected ValueError"
print("compress_level_zero_raises OK")
