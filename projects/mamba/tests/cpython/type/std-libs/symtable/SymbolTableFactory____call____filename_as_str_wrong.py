# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "symtable"
# dimension = "type"
# case = "SymbolTableFactory____call____filename_as_str_wrong"
# subject = "symtable.SymbolTableFactory.__call__(filename: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/symtable.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: symtable.SymbolTableFactory.__call__(filename: str); call it with the wrong type.

typeshed contract: filename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from symtable import SymbolTableFactory
obj = object.__new__(SymbolTableFactory)
try:
    obj.__call__(None, 12345)  # filename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
