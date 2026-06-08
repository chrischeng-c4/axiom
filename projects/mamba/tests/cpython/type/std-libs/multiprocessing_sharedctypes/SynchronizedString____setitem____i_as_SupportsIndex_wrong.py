# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "SynchronizedString____setitem____i_as_SupportsIndex_wrong"
# subject = "multiprocessing.sharedctypes.SynchronizedString.__setitem__(i: SupportsIndex)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed i"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed i
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.SynchronizedString.__setitem__(i: SupportsIndex); call it with the wrong type.

typeshed contract: i is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import SynchronizedString
obj = object.__new__(SynchronizedString)
try:
    obj.__setitem__(_W(), b"")  # i: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
