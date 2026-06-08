# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_popen_spawn_win32"
# dimension = "type"
# case = "Popen__duplicate_for_child__handle_as_int_wrong"
# subject = "multiprocessing.popen_spawn_win32.Popen.duplicate_for_child(handle: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/popen_spawn_win32.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.popen_spawn_win32.Popen.duplicate_for_child(handle: int); call it with the wrong type.

typeshed contract: handle is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.popen_spawn_win32 import Popen
obj = object.__new__(Popen)
try:
    obj.duplicate_for_child("not_an_int")  # handle: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
