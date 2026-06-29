# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_remote_debugging"
# dimension = "type"
# case = "GCMonitor__get_gc_stats__all_interpreters_as_bool_wrong"
# subject = "_remote_debugging.GCMonitor.get_gc_stats(all_interpreters: bool)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_remote_debugging.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _remote_debugging.GCMonitor.get_gc_stats(all_interpreters: bool); call it with the wrong type.

typeshed contract: all_interpreters is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _remote_debugging import GCMonitor
obj = object.__new__(GCMonitor)
try:
    obj.get_gc_stats("not_a_bool")  # all_interpreters: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
