# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_fixes_fix_xrange"
# dimension = "type"
# case = "FixXrange__finish_tree__tree_as_Node_wrong"
# subject = "lib2to3.fixes.fix_xrange.FixXrange.finish_tree(tree: Node)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/fixes/fix_xrange.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.fixes.fix_xrange.FixXrange.finish_tree(tree: Node); call it with the wrong type.

typeshed contract: tree is Node. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.fixes.fix_xrange import FixXrange
obj = object.__new__(FixXrange)
try:
    obj.finish_tree(_W(), None)  # tree: Node <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
