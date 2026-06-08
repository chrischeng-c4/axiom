# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_lsprof"
# dimension = "type"
# case = "Profiler__enable__subcalls_as_bool_wrong"
# subject = "_lsprof.Profiler.enable(subcalls: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed subcalls"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_lsprof.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed subcalls
# mamba-strict-type: TypeError
"""Type wall: _lsprof.Profiler.enable(subcalls: bool); call it with the wrong type.

typeshed contract: subcalls is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _lsprof import Profiler
obj = object.__new__(Profiler)
try:
    obj.enable("not_a_bool")  # subcalls: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
