# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "type"
# case = "CodeType____new____argcount_as_int_wrong"
# subject = "types.CodeType.__new__(argcount: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/types.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: types.CodeType.__new__(argcount: int); call it with the wrong type.

typeshed contract: argcount is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from types import CodeType
obj = object.__new__(CodeType)
try:
    obj.__new__("not_an_int", 0, 0, 0, 0, 0, b"", None, None, None, "", "", "", 0, b"", b"")  # argcount: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
