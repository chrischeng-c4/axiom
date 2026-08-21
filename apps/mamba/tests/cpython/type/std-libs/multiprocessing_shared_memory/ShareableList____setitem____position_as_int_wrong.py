# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_shared_memory"
# dimension = "type"
# case = "ShareableList____setitem____position_as_int_wrong"
# subject = "multiprocessing.shared_memory.ShareableList.__setitem__(position: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/shared_memory.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.shared_memory.ShareableList.__setitem__(position: int); call it with the wrong type.

typeshed contract: position is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.shared_memory import ShareableList
obj = object.__new__(ShareableList)
try:
    obj.__setitem__("not_an_int", None)  # position: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
