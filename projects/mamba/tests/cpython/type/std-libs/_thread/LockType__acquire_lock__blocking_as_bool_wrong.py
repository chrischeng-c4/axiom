# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_thread"
# dimension = "type"
# case = "LockType__acquire_lock__blocking_as_bool_wrong"
# subject = "_thread.LockType.acquire_lock(blocking: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed blocking"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_thread.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed blocking
# mamba-strict-type: TypeError
"""Type wall: _thread.LockType.acquire_lock(blocking: bool); call it with the wrong type.

typeshed contract: blocking is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _thread import LockType
obj = object.__new__(LockType)
try:
    obj.acquire_lock("not_a_bool")  # blocking: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
