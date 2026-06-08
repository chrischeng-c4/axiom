# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "check_constants_values"
# subject = "lzma.CHECK_NONE"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""lzma.CHECK_NONE: CHECK_NONE/CRC32/CRC64/SHA256 equal CPython's 0/1/4/10"""
import lzma


assert lzma.CHECK_NONE == 0, f"CHECK_NONE = {lzma.CHECK_NONE!r}"
assert lzma.CHECK_CRC32 == 1, f"CHECK_CRC32 = {lzma.CHECK_CRC32!r}"
assert lzma.CHECK_CRC64 == 4, f"CHECK_CRC64 = {lzma.CHECK_CRC64!r}"
assert lzma.CHECK_SHA256 == 10, f"CHECK_SHA256 = {lzma.CHECK_SHA256!r}"
print("check_constants_values OK")
