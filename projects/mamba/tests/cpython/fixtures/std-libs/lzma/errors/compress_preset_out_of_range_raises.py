# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "errors"
# case = "compress_preset_out_of_range_raises"
# subject = "lzma.compress"
# kind = "mechanical"
# xfail = "lzma.compress ignores the preset kwarg; preset=10 does not raise (src/runtime/stdlib/lzma_mod.rs:153)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.compress: compress_preset_out_of_range_raises (errors)."""
import lzma

_raised = False
try:
    lzma.compress(b'x', preset=10)
except lzma.LZMAError:
    _raised = True
assert _raised, "compress_preset_out_of_range_raises: expected lzma.LZMAError"
print("compress_preset_out_of_range_raises OK")
