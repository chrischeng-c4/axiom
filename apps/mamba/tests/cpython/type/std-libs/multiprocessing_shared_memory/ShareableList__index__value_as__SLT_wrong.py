# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_shared_memory"
# dimension = "type"
# case = "ShareableList__index__value_as__SLT_wrong"
# subject = "multiprocessing.shared_memory.ShareableList.index(value: _SLT)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed value"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/shared_memory.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed value
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.shared_memory.ShareableList.index(value: _SLT); call it with the wrong type.

typeshed contract: value is _SLT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.shared_memory import ShareableList
obj = object.__new__(ShareableList)
try:
    obj.index(_W())  # value: _SLT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
