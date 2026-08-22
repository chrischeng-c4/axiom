use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/multiprocessing_queues/JoinableQueue____setstate____state_as__JoinableQueueState_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_queues_JoinableQueue____setstate____state_as__JoinableQueueState_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_queues"
# dimension = "type"
# case = "JoinableQueue____setstate____state_as__JoinableQueueState_wrong"
# subject = "multiprocessing.queues.JoinableQueue.__setstate__(state: _JoinableQueueState)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/queues.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.queues.JoinableQueue.__setstate__(state: _JoinableQueueState); call it with the wrong type.

typeshed contract: state is _JoinableQueueState. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.queues import JoinableQueue
obj = object.__new__(JoinableQueue)
try:
    obj.__setstate__(_W())  # state: _JoinableQueueState <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_queues/Queue____setstate____state_as__QueueState_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_queues_Queue____setstate____state_as__QueueState_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_queues"
# dimension = "type"
# case = "Queue____setstate____state_as__QueueState_wrong"
# subject = "multiprocessing.queues.Queue.__setstate__(state: _QueueState)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/queues.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.queues.Queue.__setstate__(state: _QueueState); call it with the wrong type.

typeshed contract: state is _QueueState. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.queues import Queue
obj = object.__new__(Queue)
try:
    obj.__setstate__(_W())  # state: _QueueState <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_queues/Queue__init__maxsize_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_queues_Queue__init__maxsize_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_queues"
# dimension = "type"
# case = "Queue__init__maxsize_as_int_wrong"
# subject = "multiprocessing.queues.Queue.__init__(maxsize: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/queues.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.queues.Queue.__init__(maxsize: int); call it with the wrong type.

typeshed contract: maxsize is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.queues import Queue
try:
    Queue("not_an_int")  # maxsize: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_queues/SimpleQueue____setstate____state_as__SimpleQueueState_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_queues_SimpleQueue____setstate____state_as__SimpleQueueState_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_queues"
# dimension = "type"
# case = "SimpleQueue____setstate____state_as__SimpleQueueState_wrong"
# subject = "multiprocessing.queues.SimpleQueue.__setstate__(state: _SimpleQueueState)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/queues.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.queues.SimpleQueue.__setstate__(state: _SimpleQueueState); call it with the wrong type.

typeshed contract: state is _SimpleQueueState. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.queues import SimpleQueue
obj = object.__new__(SimpleQueue)
try:
    obj.__setstate__(_W())  # state: _SimpleQueueState <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
