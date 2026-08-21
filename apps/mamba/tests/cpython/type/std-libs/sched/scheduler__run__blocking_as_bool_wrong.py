# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sched"
# dimension = "type"
# case = "scheduler__run__blocking_as_bool_wrong"
# subject = "sched.scheduler.run(blocking: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed blocking"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sched.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed blocking
# mamba-strict-type: TypeError
"""Type wall: sched.scheduler.run(blocking: bool); call it with the wrong type.

typeshed contract: blocking is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sched import scheduler
obj = object.__new__(scheduler)
try:
    obj.run("not_a_bool")  # blocking: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
