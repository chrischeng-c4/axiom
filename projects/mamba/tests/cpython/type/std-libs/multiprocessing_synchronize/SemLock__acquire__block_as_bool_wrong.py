# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_synchronize"
# dimension = "type"
# case = "SemLock__acquire__block_as_bool_wrong"
# subject = "multiprocessing.synchronize.SemLock.acquire(block: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed block"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/synchronize.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed block
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.synchronize.SemLock.acquire(block: bool); call it with the wrong type.

typeshed contract: block is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.synchronize import SemLock
obj = object.__new__(SemLock)
try:
    obj.acquire("not_a_bool")  # block: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
