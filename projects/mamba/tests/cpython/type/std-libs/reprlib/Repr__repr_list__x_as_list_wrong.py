# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "reprlib"
# dimension = "type"
# case = "Repr__repr_list__x_as_list_wrong"
# subject = "reprlib.Repr.repr_list(x: list)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed x"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/reprlib.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed x
# mamba-strict-type: TypeError
"""Type wall: reprlib.Repr.repr_list(x: list); call it with the wrong type.

typeshed contract: x is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from reprlib import Repr
obj = object.__new__(Repr)
try:
    obj.repr_list(12345, 0)  # x: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
