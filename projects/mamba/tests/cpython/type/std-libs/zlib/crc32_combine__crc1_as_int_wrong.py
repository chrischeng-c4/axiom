# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "type"
# case = "crc32_combine__crc1_as_int_wrong"
# subject = "zlib.crc32_combine(crc1: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zlib.crc32_combine(crc1: int); call it with the wrong type.

typeshed contract: crc1 is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zlib import crc32_combine
try:
    crc32_combine("not_an_int", 0, 0)  # crc1: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
