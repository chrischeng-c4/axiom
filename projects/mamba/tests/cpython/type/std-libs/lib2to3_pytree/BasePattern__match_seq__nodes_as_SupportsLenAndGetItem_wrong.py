# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pytree"
# dimension = "type"
# case = "BasePattern__match_seq__nodes_as_SupportsLenAndGetItem_wrong"
# subject = "lib2to3.pytree.BasePattern.match_seq(nodes: SupportsLenAndGetItem)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pytree.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pytree.BasePattern.match_seq(nodes: SupportsLenAndGetItem); call it with the wrong type.

typeshed contract: nodes is SupportsLenAndGetItem. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.pytree import BasePattern
obj = object.__new__(BasePattern)
try:
    obj.match_seq(_W())  # nodes: SupportsLenAndGetItem <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
