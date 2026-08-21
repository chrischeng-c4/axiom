# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "type"
# case = "TopologicalSorter__add__node_as__T_wrong"
# subject = "graphlib.TopologicalSorter.add(node: _T)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed node"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/graphlib.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed node
# mamba-strict-type: TypeError
"""Type wall: graphlib.TopologicalSorter.add(node: _T); call it with the wrong type.

typeshed contract: node is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from graphlib import TopologicalSorter
obj = object.__new__(TopologicalSorter)
try:
    obj.add(_W())  # node: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
