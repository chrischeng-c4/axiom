# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "type"
# case = "fix_missing_locations__node_as__T_wrong"
# subject = "ast.fix_missing_locations(node: _T)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ast.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ast.fix_missing_locations(node: _T); call it with the wrong type.

typeshed contract: node is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ast import fix_missing_locations
try:
    fix_missing_locations(_W())  # node: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
