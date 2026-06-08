# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "errors"
# case = "decompress_bad_input_raises"
# subject = "lzma.decompress"
# kind = "mechanical"
# xfail = "lzma.decompress returns empty bytes on bad input instead of raising LZMAError (src/runtime/stdlib/lzma_mod.rs:179-188)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.decompress: decompress_bad_input_raises (errors)."""
import lzma

_raised = False
try:
    lzma.decompress(b'not lzma data')
except lzma.LZMAError:
    _raised = True
assert _raised, "decompress_bad_input_raises: expected lzma.LZMAError"
print("decompress_bad_input_raises OK")
