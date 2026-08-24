use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/asyncio_unix_events/AbstractChildWatcher__attach_loop__loop_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_unix_events_AbstractChildWatcher__attach_loop__loop_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_unix_events"
# dimension = "type"
# case = "AbstractChildWatcher__attach_loop__loop_as_typed_wrong"
# subject = "asyncio.unix_events.AbstractChildWatcher.attach_loop(loop: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/unix_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.unix_events.AbstractChildWatcher.attach_loop(loop: typed); call it with the wrong type.

typeshed contract: loop is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.unix_events import AbstractChildWatcher
obj = object.__new__(AbstractChildWatcher)
try:
    obj.attach_loop(_W())  # loop: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_unix_events/AbstractChildWatcher__remove_child_handler__pid_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_unix_events_AbstractChildWatcher__remove_child_handler__pid_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_unix_events"
# dimension = "type"
# case = "AbstractChildWatcher__remove_child_handler__pid_as_int_wrong"
# subject = "asyncio.unix_events.AbstractChildWatcher.remove_child_handler(pid: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/unix_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.unix_events.AbstractChildWatcher.remove_child_handler(pid: int); call it with the wrong type.

typeshed contract: pid is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.unix_events import AbstractChildWatcher
obj = object.__new__(AbstractChildWatcher)
try:
    obj.remove_child_handler("not_an_int")  # pid: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_unix_events/BaseChildWatcher__attach_loop__loop_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_unix_events_BaseChildWatcher__attach_loop__loop_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_unix_events"
# dimension = "type"
# case = "BaseChildWatcher__attach_loop__loop_as_typed_wrong"
# subject = "asyncio.unix_events.BaseChildWatcher.attach_loop(loop: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/unix_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.unix_events.BaseChildWatcher.attach_loop(loop: typed); call it with the wrong type.

typeshed contract: loop is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.unix_events import BaseChildWatcher
obj = object.__new__(BaseChildWatcher)
try:
    obj.attach_loop(_W())  # loop: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_unix_events/FastChildWatcher__remove_child_handler__pid_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_unix_events_FastChildWatcher__remove_child_handler__pid_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_unix_events"
# dimension = "type"
# case = "FastChildWatcher__remove_child_handler__pid_as_int_wrong"
# subject = "asyncio.unix_events.FastChildWatcher.remove_child_handler(pid: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/unix_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.unix_events.FastChildWatcher.remove_child_handler(pid: int); call it with the wrong type.

typeshed contract: pid is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.unix_events import FastChildWatcher
obj = object.__new__(FastChildWatcher)
try:
    obj.remove_child_handler("not_an_int")  # pid: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_unix_events/MultiLoopChildWatcher__attach_loop__loop_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_unix_events_MultiLoopChildWatcher__attach_loop__loop_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_unix_events"
# dimension = "type"
# case = "MultiLoopChildWatcher__attach_loop__loop_as_typed_wrong"
# subject = "asyncio.unix_events.MultiLoopChildWatcher.attach_loop(loop: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/unix_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.unix_events.MultiLoopChildWatcher.attach_loop(loop: typed); call it with the wrong type.

typeshed contract: loop is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.unix_events import MultiLoopChildWatcher
obj = object.__new__(MultiLoopChildWatcher)
try:
    obj.attach_loop(_W())  # loop: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_unix_events/MultiLoopChildWatcher__remove_child_handler__pid_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_unix_events_MultiLoopChildWatcher__remove_child_handler__pid_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_unix_events"
# dimension = "type"
# case = "MultiLoopChildWatcher__remove_child_handler__pid_as_int_wrong"
# subject = "asyncio.unix_events.MultiLoopChildWatcher.remove_child_handler(pid: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/unix_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.unix_events.MultiLoopChildWatcher.remove_child_handler(pid: int); call it with the wrong type.

typeshed contract: pid is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.unix_events import MultiLoopChildWatcher
obj = object.__new__(MultiLoopChildWatcher)
try:
    obj.remove_child_handler("not_an_int")  # pid: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_unix_events/PidfdChildWatcher__attach_loop__loop_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_unix_events_PidfdChildWatcher__attach_loop__loop_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_unix_events"
# dimension = "type"
# case = "PidfdChildWatcher__attach_loop__loop_as_typed_wrong"
# subject = "asyncio.unix_events.PidfdChildWatcher.attach_loop(loop: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/unix_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.unix_events.PidfdChildWatcher.attach_loop(loop: typed); call it with the wrong type.

typeshed contract: loop is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.unix_events import PidfdChildWatcher
obj = object.__new__(PidfdChildWatcher)
try:
    obj.attach_loop(_W())  # loop: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_unix_events/PidfdChildWatcher__remove_child_handler__pid_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_unix_events_PidfdChildWatcher__remove_child_handler__pid_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_unix_events"
# dimension = "type"
# case = "PidfdChildWatcher__remove_child_handler__pid_as_int_wrong"
# subject = "asyncio.unix_events.PidfdChildWatcher.remove_child_handler(pid: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/unix_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.unix_events.PidfdChildWatcher.remove_child_handler(pid: int); call it with the wrong type.

typeshed contract: pid is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.unix_events import PidfdChildWatcher
obj = object.__new__(PidfdChildWatcher)
try:
    obj.remove_child_handler("not_an_int")  # pid: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_unix_events/SafeChildWatcher__remove_child_handler__pid_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_unix_events_SafeChildWatcher__remove_child_handler__pid_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_unix_events"
# dimension = "type"
# case = "SafeChildWatcher__remove_child_handler__pid_as_int_wrong"
# subject = "asyncio.unix_events.SafeChildWatcher.remove_child_handler(pid: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/unix_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.unix_events.SafeChildWatcher.remove_child_handler(pid: int); call it with the wrong type.

typeshed contract: pid is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.unix_events import SafeChildWatcher
obj = object.__new__(SafeChildWatcher)
try:
    obj.remove_child_handler("not_an_int")  # pid: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_unix_events/ThreadedChildWatcher__attach_loop__loop_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_unix_events_ThreadedChildWatcher__attach_loop__loop_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_unix_events"
# dimension = "type"
# case = "ThreadedChildWatcher__attach_loop__loop_as_typed_wrong"
# subject = "asyncio.unix_events.ThreadedChildWatcher.attach_loop(loop: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/unix_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.unix_events.ThreadedChildWatcher.attach_loop(loop: typed); call it with the wrong type.

typeshed contract: loop is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.unix_events import ThreadedChildWatcher
obj = object.__new__(ThreadedChildWatcher)
try:
    obj.attach_loop(_W())  # loop: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_unix_events/ThreadedChildWatcher__remove_child_handler__pid_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_unix_events_ThreadedChildWatcher__remove_child_handler__pid_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_unix_events"
# dimension = "type"
# case = "ThreadedChildWatcher__remove_child_handler__pid_as_int_wrong"
# subject = "asyncio.unix_events.ThreadedChildWatcher.remove_child_handler(pid: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/unix_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.unix_events.ThreadedChildWatcher.remove_child_handler(pid: int); call it with the wrong type.

typeshed contract: pid is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.unix_events import ThreadedChildWatcher
obj = object.__new__(ThreadedChildWatcher)
try:
    obj.remove_child_handler("not_an_int")  # pid: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
