# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "named_apis_callable_constants_not"
# subject = "lzma.compress"
# kind = "semantic"
# xfail = "lzma.open / LZMAFile are sentinel strings, not callable (src/runtime/stdlib/lzma_mod.rs:79-82)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.compress: compress/decompress/open/LZMAFile are callable while FORMAT_XZ and CHECK_CRC64 are plain ints (not callable)"""
import lzma


assert callable(lzma.compress) and callable(lzma.decompress), "fns callable"
assert callable(lzma.LZMAFile) and callable(lzma.open), "class/open callable"
assert not callable(lzma.FORMAT_XZ), "FORMAT_XZ is not callable"
assert not callable(lzma.CHECK_CRC64), "CHECK_CRC64 is not callable"
print("named_apis_callable_constants_not OK")
