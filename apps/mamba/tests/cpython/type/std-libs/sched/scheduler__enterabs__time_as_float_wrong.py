# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sched"
# dimension = "type"
# case = "scheduler__enterabs__time_as_float_wrong"
# subject = "sched.scheduler.enterabs(time: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sched.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sched.scheduler.enterabs(time: float); call it with the wrong type.

typeshed contract: time is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sched import scheduler
obj = object.__new__(scheduler)
try:
    obj.enterabs("not_a_float", None, None)  # time: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
