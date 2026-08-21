# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sched"
# dimension = "type"
# case = "scheduler__init__timefunc_as_Callable_wrong"
# subject = "sched.scheduler.__init__(timefunc: Callable)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed timefunc"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sched.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed timefunc
# mamba-strict-type: TypeError
"""Type wall: sched.scheduler.__init__(timefunc: Callable); call it with the wrong type.

typeshed contract: timefunc is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from sched import scheduler
try:
    scheduler(_W())  # timefunc: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
