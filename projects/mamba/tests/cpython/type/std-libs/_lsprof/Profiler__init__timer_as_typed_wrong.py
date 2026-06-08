# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_lsprof"
# dimension = "type"
# case = "Profiler__init__timer_as_typed_wrong"
# subject = "_lsprof.Profiler.__init__(timer: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed timer"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_lsprof.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed timer
# mamba-strict-type: TypeError
"""Type wall: _lsprof.Profiler.__init__(timer: typed); call it with the wrong type.

typeshed contract: timer is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _lsprof import Profiler
try:
    Profiler(_W())  # timer: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
