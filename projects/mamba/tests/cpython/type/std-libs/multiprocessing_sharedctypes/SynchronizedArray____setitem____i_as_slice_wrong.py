# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "SynchronizedArray____setitem____i_as_slice_wrong"
# subject = "multiprocessing.sharedctypes.SynchronizedArray.__setitem__(i: slice)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed i"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed i
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.SynchronizedArray.__setitem__(i: slice); call it with the wrong type.

typeshed contract: i is slice. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import SynchronizedArray
obj = object.__new__(SynchronizedArray)
try:
    obj.__setitem__(_W(), None)  # i: slice <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
