# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "type"
# case = "SpooledTemporaryFile__read__n_as_int_wrong"
# subject = "tempfile.SpooledTemporaryFile.read(n: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tempfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tempfile.SpooledTemporaryFile.read(n: int); call it with the wrong type.

typeshed contract: n is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tempfile import SpooledTemporaryFile
obj = object.__new__(SpooledTemporaryFile)
try:
    obj.read("not_an_int")  # n: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
