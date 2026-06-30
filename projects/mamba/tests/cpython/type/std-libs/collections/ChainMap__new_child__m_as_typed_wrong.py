# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "type"
# case = "ChainMap__new_child__m_as_typed_wrong"
# subject = "collections.ChainMap.new_child(m: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/collections.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: collections.ChainMap.new_child(m: typed); call it with the wrong type.

typeshed contract: m is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from collections import ChainMap
obj = object.__new__(ChainMap)
try:
    obj.new_child(_W())  # m: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
