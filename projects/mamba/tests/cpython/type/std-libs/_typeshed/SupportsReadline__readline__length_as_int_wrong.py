# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_typeshed"
# dimension = "type"
# case = "SupportsReadline__readline__length_as_int_wrong"
# subject = "_typeshed.SupportsReadline.readline(length: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_typeshed.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _typeshed.SupportsReadline.readline(length: int); call it with the wrong type.

typeshed contract: length is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _typeshed import SupportsReadline
obj = object.__new__(SupportsReadline)
try:
    obj.readline("not_an_int")  # length: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
