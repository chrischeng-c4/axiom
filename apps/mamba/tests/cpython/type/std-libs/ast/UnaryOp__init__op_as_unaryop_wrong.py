# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "type"
# case = "UnaryOp__init__op_as_unaryop_wrong"
# subject = "ast.UnaryOp.__init__(op: unaryop)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ast.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ast.UnaryOp.__init__(op: unaryop); call it with the wrong type.

typeshed contract: op is unaryop. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ast import UnaryOp
try:
    UnaryOp(_W(), None)  # op: unaryop <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
