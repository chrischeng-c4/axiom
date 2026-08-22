use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/sched/scheduler__cancel__event_as_Event_wrong.py`.
#[test]
fn test_gen_type_std_libs_sched_scheduler__cancel__event_as_Event_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
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
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/sched/scheduler__enter__delay_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_sched_scheduler__enter__delay_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sched"
# dimension = "type"
# case = "scheduler__enter__delay_as_float_wrong"
# subject = "sched.scheduler.enter(delay: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sched.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sched.scheduler.enter(delay: float); call it with the wrong type.

typeshed contract: delay is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sched import scheduler
obj = object.__new__(scheduler)
try:
    obj.enter("not_a_float", None, None)  # delay: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/sched/scheduler__enterabs__time_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_sched_scheduler__enterabs__time_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
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
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
