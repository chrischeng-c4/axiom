# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "errors"
# case = "decompress_after_eof_raises"
# subject = "bz2.BZ2Decompressor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2Decompressor: decompress_after_eof_raises (errors)."""
import bz2

_raised = False
try:
    (lambda d: (d.decompress(bz2.compress(b"x")), d.decompress(b"more"))) (bz2.BZ2Decompressor())
except EOFError:
    _raised = True
assert _raised, "decompress_after_eof_raises: expected EOFError"
print("decompress_after_eof_raises OK")
