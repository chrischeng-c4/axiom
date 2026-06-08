# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "reprlib"
# dimension = "type"
# case = "recursive_repr__fillvalue_as_str_wrong"
# subject = "reprlib.recursive_repr(fillvalue: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/reprlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: reprlib.recursive_repr(fillvalue: str); call it with the wrong type.

typeshed contract: fillvalue is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from reprlib import recursive_repr
try:
    recursive_repr(12345)  # fillvalue: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
