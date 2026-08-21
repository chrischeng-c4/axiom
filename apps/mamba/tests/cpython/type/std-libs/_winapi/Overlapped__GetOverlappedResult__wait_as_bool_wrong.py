# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_winapi"
# dimension = "type"
# case = "Overlapped__GetOverlappedResult__wait_as_bool_wrong"
# subject = "_winapi.Overlapped.GetOverlappedResult(wait: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed wait"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_winapi.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed wait
# mamba-strict-type: TypeError
"""Type wall: _winapi.Overlapped.GetOverlappedResult(wait: bool); call it with the wrong type.

typeshed contract: wait is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _winapi import Overlapped
obj = object.__new__(Overlapped)
try:
    obj.GetOverlappedResult("not_a_bool")  # wait: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
