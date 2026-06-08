# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "type"
# case = "GzipFile__flush__zlib_mode_as_int_wrong"
# subject = "gzip.GzipFile.flush(zlib_mode: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/gzip.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: gzip.GzipFile.flush(zlib_mode: int); call it with the wrong type.

typeshed contract: zlib_mode is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from gzip import GzipFile
obj = object.__new__(GzipFile)
try:
    obj.flush("not_an_int")  # zlib_mode: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
