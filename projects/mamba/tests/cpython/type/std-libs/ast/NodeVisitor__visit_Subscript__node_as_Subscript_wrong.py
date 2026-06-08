# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "type"
# case = "NodeVisitor__visit_Subscript__node_as_Subscript_wrong"
# subject = "ast.NodeVisitor.visit_Subscript(node: Subscript)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ast.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ast.NodeVisitor.visit_Subscript(node: Subscript); call it with the wrong type.

typeshed contract: node is Subscript. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ast import NodeVisitor
obj = object.__new__(NodeVisitor)
try:
    obj.visit_Subscript(_W())  # node: Subscript <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
