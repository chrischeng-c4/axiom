# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "reprlib"
# dimension = "type"
# case = "Repr__repr_int__x_as_int_wrong"
# subject = "reprlib.Repr.repr_int(x: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/reprlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: reprlib.Repr.repr_int(x: int); call it with the wrong type.

typeshed contract: x is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from reprlib import Repr
obj = object.__new__(Repr)
try:
    obj.repr_int("not_an_int", 0)  # x: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
