# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "errors"
# case = "decompressor_ctor_positional_raises"
# subject = "bz2.BZ2Decompressor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2Decompressor: decompressor_ctor_positional_raises (errors)."""
import bz2

_raised = False
try:
    bz2.BZ2Decompressor(42)
except TypeError:
    _raised = True
assert _raised, "decompressor_ctor_positional_raises: expected TypeError"
print("decompressor_ctor_positional_raises OK")
