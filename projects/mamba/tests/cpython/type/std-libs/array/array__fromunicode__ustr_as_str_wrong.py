# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "type"
# case = "array__fromunicode__ustr_as_str_wrong"
# subject = "array.array.fromunicode(ustr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/array.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: array.array.fromunicode(ustr: str); call it with the wrong type.

typeshed contract: ustr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from array import array
obj = object.__new__(array)
try:
    obj.fromunicode(12345)  # ustr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
