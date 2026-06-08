# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "maxsize_platform_word"
# subject = "sys.maxsize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.maxsize: sys.maxsize is the platform pointer-word signed max: 2**63-1 on 64-bit (struct.calcsize('P')*8 == 64), else 2**31-1"""
import sys
import struct

_bits = struct.calcsize("P") * 8
if _bits == 64:
    assert sys.maxsize == 2**63 - 1, f"64-bit maxsize = {sys.maxsize!r}"
else:
    assert sys.maxsize == 2**31 - 1, f"32-bit maxsize = {sys.maxsize!r}"
print("maxsize_platform_word OK")
