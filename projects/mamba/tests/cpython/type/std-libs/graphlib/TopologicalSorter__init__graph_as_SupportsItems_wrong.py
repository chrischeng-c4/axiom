# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "type"
# case = "TopologicalSorter__init__graph_as_SupportsItems_wrong"
# subject = "graphlib.TopologicalSorter.__init__(graph: SupportsItems)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed graph"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/graphlib.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed graph
# mamba-strict-type: TypeError
"""Type wall: graphlib.TopologicalSorter.__init__(graph: SupportsItems); call it with the wrong type.

typeshed contract: graph is SupportsItems. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from graphlib import TopologicalSorter
try:
    TopologicalSorter(_W())  # graph: SupportsItems <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
