# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "errors"
# case = "lzmafile_bad_mode_raises"
# subject = "lzma.LZMAFile"
# kind = "mechanical"
# xfail = "lzma.LZMAFile is a sentinel-string stub; constructing it does not raise (src/runtime/stdlib/lzma_mod.rs:79-80)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.LZMAFile: lzmafile_bad_mode_raises (errors)."""
import lzma

_raised = False
try:
    lzma.LZMAFile(__import__('io').BytesIO(lzma.compress(b'x')), 'rt')
except ValueError:
    _raised = True
assert _raised, "lzmafile_bad_mode_raises: expected ValueError"
print("lzmafile_bad_mode_raises OK")
