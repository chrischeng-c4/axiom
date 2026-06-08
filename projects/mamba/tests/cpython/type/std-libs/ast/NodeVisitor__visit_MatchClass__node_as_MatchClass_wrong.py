# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "type"
# case = "NodeVisitor__visit_MatchClass__node_as_MatchClass_wrong"
# subject = "ast.NodeVisitor.visit_MatchClass(node: MatchClass)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ast.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ast.NodeVisitor.visit_MatchClass(node: MatchClass); call it with the wrong type.

typeshed contract: node is MatchClass. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ast import NodeVisitor
obj = object.__new__(NodeVisitor)
try:
    obj.visit_MatchClass(_W())  # node: MatchClass <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
