# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "type"
# case = "BinOp__init__left_as_expr_wrong"
# subject = "ast.BinOp.__init__(left: expr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ast.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ast.BinOp.__init__(left: expr); call it with the wrong type.

typeshed contract: left is expr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ast import BinOp
try:
    BinOp(_W(), None, None)  # left: expr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
