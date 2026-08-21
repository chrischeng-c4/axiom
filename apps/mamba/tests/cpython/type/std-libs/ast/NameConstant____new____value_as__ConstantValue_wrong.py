# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "type"
# case = "NameConstant____new____value_as__ConstantValue_wrong"
# subject = "ast.NameConstant.__new__(value: _ConstantValue)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ast.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ast.NameConstant.__new__(value: _ConstantValue); call it with the wrong type.

typeshed contract: value is _ConstantValue. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ast import NameConstant
try:
    obj = object.__new__(NameConstant)
    obj.__new__(_W(), None)  # value: _ConstantValue <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
