# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "symtable"
# dimension = "type"
# case = "symtable__code_as_str_wrong"
# subject = "symtable.symtable(code: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/symtable.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: symtable.symtable(code: str); call it with the wrong type.

typeshed contract: code is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from symtable import symtable
try:
    symtable(12345, "", "")  # code: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
