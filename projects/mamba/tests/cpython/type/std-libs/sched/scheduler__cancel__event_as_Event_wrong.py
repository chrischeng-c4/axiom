# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sched"
# dimension = "type"
# case = "scheduler__cancel__event_as_Event_wrong"
# subject = "sched.scheduler.cancel(event: Event)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sched.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sched.scheduler.cancel(event: Event); call it with the wrong type.

typeshed contract: event is Event. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from sched import scheduler
obj = object.__new__(scheduler)
try:
    obj.cancel(_W())  # event: Event <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
