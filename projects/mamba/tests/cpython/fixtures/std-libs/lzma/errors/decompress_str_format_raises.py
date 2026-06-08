# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "errors"
# case = "decompress_str_format_raises"
# subject = "lzma.decompress"
# kind = "mechanical"
# xfail = "lzma.decompress ignores the format kwarg; a str format does not raise TypeError (src/runtime/stdlib/lzma_mod.rs:179)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.decompress: decompress_str_format_raises (errors)."""
import lzma

_raised = False
try:
    lzma.decompress(b'', format='lzma')
except TypeError:
    _raised = True
assert _raised, "decompress_str_format_raises: expected TypeError"
print("decompress_str_format_raises OK")
