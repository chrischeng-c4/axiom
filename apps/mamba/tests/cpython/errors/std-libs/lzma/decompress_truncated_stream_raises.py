# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "errors"
# case = "decompress_truncated_stream_raises"
# subject = "lzma.decompress"
# kind = "mechanical"
# xfail = "lzma.decompress returns empty bytes on a truncated stream instead of raising (src/runtime/stdlib/lzma_mod.rs:179-188)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.decompress: decompress_truncated_stream_raises (errors)."""
import lzma

_raised = False
try:
    lzma.decompress(lzma.compress(b'hello lzma')[:4])
except lzma.LZMAError:
    _raised = True
assert _raised, "decompress_truncated_stream_raises: expected lzma.LZMAError"
print("decompress_truncated_stream_raises OK")
