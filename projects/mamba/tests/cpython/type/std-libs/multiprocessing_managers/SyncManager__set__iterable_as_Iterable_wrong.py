# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_managers"
# dimension = "type"
# case = "SyncManager__set__iterable_as_Iterable_wrong"
# subject = "multiprocessing.managers.SyncManager.set(iterable: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/managers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.managers.SyncManager.set(iterable: Iterable); call it with the wrong type.

typeshed contract: iterable is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.managers import SyncManager
obj = object.__new__(SyncManager)
try:
    obj.set(_W())  # iterable: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
