# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "symtable"
# dimension = "type"
# case = "SymbolTable__init__filename_as_str_wrong"
# subject = "symtable.SymbolTable.__init__(filename: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/symtable.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: symtable.SymbolTable.__init__(filename: str); call it with the wrong type.

typeshed contract: filename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from symtable import SymbolTable
try:
    SymbolTable(None, 12345)  # filename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
