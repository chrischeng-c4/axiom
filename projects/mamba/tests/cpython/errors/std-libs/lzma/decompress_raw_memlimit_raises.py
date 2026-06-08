# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "errors"
# case = "decompress_raw_memlimit_raises"
# subject = "lzma.decompress"
# kind = "mechanical"
# xfail = "lzma.decompress ignores format/memlimit kwargs; FORMAT_RAW without filters does not raise (src/runtime/stdlib/lzma_mod.rs:179)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.decompress: decompress_raw_memlimit_raises (errors)."""
import lzma

_raised = False
try:
    lzma.decompress(b'', format=lzma.FORMAT_RAW, memlimit=1 << 24)
except ValueError:
    _raised = True
assert _raised, "decompress_raw_memlimit_raises: expected ValueError"
print("decompress_raw_memlimit_raises OK")
