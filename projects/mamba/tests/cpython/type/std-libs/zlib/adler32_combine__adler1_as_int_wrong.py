# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "type"
# case = "adler32_combine__adler1_as_int_wrong"
# subject = "zlib.adler32_combine(adler1: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zlib.adler32_combine(adler1: int); call it with the wrong type.

typeshed contract: adler1 is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zlib import adler32_combine
try:
    adler32_combine("not_an_int", 0, 0)  # adler1: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
