# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_remote_debugging"
# dimension = "type"
# case = "get_child_pids__pid_as_int_wrong"
# subject = "_remote_debugging.get_child_pids(pid: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_remote_debugging.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _remote_debugging.get_child_pids(pid: int); call it with the wrong type.

typeshed contract: pid is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _remote_debugging import get_child_pids
try:
    get_child_pids("not_an_int")  # pid: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
