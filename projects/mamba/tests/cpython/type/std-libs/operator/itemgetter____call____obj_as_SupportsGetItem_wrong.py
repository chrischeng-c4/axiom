# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "type"
# case = "itemgetter____call____obj_as_SupportsGetItem_wrong"
# subject = "operator.itemgetter.__call__(obj: SupportsGetItem)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: operator.itemgetter.__call__(obj: SupportsGetItem); call it with the wrong type.

typeshed contract: obj is SupportsGetItem. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from operator import itemgetter
obj = object.__new__(itemgetter)
try:
    obj.__call__(_W())  # obj: SupportsGetItem <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
