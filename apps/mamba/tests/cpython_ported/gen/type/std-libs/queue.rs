use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_queue/SimpleQueue__get__block_as_bool_wrong.py`.
#[test]
fn test_gen_type_std_libs__queue_SimpleQueue__get__block_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_queue"
# dimension = "type"
# case = "SimpleQueue__get__block_as_bool_wrong"
# subject = "_queue.SimpleQueue.get(block: bool)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_queue.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _queue.SimpleQueue.get(block: bool); call it with the wrong type.

typeshed contract: block is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _queue import SimpleQueue
obj = object.__new__(SimpleQueue)
try:
    obj.get("not_a_bool")  # block: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_queue/SimpleQueue__put__item_as__T_wrong.py`.
#[test]
fn test_gen_type_std_libs__queue_SimpleQueue__put__item_as__T_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_queue"
# dimension = "type"
# case = "SimpleQueue__put__item_as__T_wrong"
# subject = "_queue.SimpleQueue.put(item: _T)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_queue.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _queue.SimpleQueue.put(item: _T); call it with the wrong type.

typeshed contract: item is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _queue import SimpleQueue
obj = object.__new__(SimpleQueue)
try:
    obj.put(_W())  # item: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_queue/SimpleQueue__put_nowait__item_as__T_wrong.py`.
#[test]
fn test_gen_type_std_libs__queue_SimpleQueue__put_nowait__item_as__T_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_queue"
# dimension = "type"
# case = "SimpleQueue__put_nowait__item_as__T_wrong"
# subject = "_queue.SimpleQueue.put_nowait(item: _T)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_queue.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _queue.SimpleQueue.put_nowait(item: _T); call it with the wrong type.

typeshed contract: item is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _queue import SimpleQueue
obj = object.__new__(SimpleQueue)
try:
    obj.put_nowait(_W())  # item: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/queue/Queue__get__block_as_bool_wrong.py`.
#[test]
fn test_gen_type_std_libs_queue_Queue__get__block_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "type"
# case = "Queue__get__block_as_bool_wrong"
# subject = "queue.Queue.get(block: bool)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/queue.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: queue.Queue.get(block: bool); call it with the wrong type.

typeshed contract: block is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from queue import Queue
obj = object.__new__(Queue)
try:
    obj.get("not_a_bool")  # block: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/queue/Queue__init__maxsize_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_queue_Queue__init__maxsize_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "type"
# case = "Queue__init__maxsize_as_int_wrong"
# subject = "queue.Queue.__init__(maxsize: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/queue.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: queue.Queue.__init__(maxsize: int); call it with the wrong type.

typeshed contract: maxsize is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from queue import Queue
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

/// Ported from `tests/cpython/type/std-libs/queue/Queue__put__item_as__T_wrong.py`.
#[test]
fn test_gen_type_std_libs_queue_Queue__put__item_as__T_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "type"
# case = "Queue__put__item_as__T_wrong"
# subject = "queue.Queue.put(item: _T)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/queue.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: queue.Queue.put(item: _T); call it with the wrong type.

typeshed contract: item is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from queue import Queue
obj = object.__new__(Queue)
try:
    obj.put(_W())  # item: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/queue/Queue__put_nowait__item_as__T_wrong.py`.
#[test]
fn test_gen_type_std_libs_queue_Queue__put_nowait__item_as__T_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "type"
# case = "Queue__put_nowait__item_as__T_wrong"
# subject = "queue.Queue.put_nowait(item: _T)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/queue.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: queue.Queue.put_nowait(item: _T); call it with the wrong type.

typeshed contract: item is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from queue import Queue
obj = object.__new__(Queue)
try:
    obj.put_nowait(_W())  # item: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/queue/Queue__shutdown__immediate_as_bool_wrong.py`.
#[test]
fn test_gen_type_std_libs_queue_Queue__shutdown__immediate_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "type"
# case = "Queue__shutdown__immediate_as_bool_wrong"
# subject = "queue.Queue.shutdown(immediate: bool)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/queue.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: queue.Queue.shutdown(immediate: bool); call it with the wrong type.

typeshed contract: immediate is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from queue import Queue
obj = object.__new__(Queue)
try:
    obj.shutdown("not_a_bool")  # immediate: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
