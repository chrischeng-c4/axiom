use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/concurrent_futures__base/Future__exception__timeout_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_concurrent_futures__base_Future__exception__timeout_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures__base"
# dimension = "type"
# case = "Future__exception__timeout_as_typed_wrong"
# subject = "concurrent.futures._base.Future.exception(timeout: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/futures/_base.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.futures._base.Future.exception(timeout: typed); call it with the wrong type.

typeshed contract: timeout is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.futures._base import Future
obj = object.__new__(Future)
try:
    obj.exception(_W())  # timeout: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/concurrent_futures__base/Future__result__timeout_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_concurrent_futures__base_Future__result__timeout_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures__base"
# dimension = "type"
# case = "Future__result__timeout_as_typed_wrong"
# subject = "concurrent.futures._base.Future.result(timeout: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/futures/_base.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.futures._base.Future.result(timeout: typed); call it with the wrong type.

typeshed contract: timeout is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.futures._base import Future
obj = object.__new__(Future)
try:
    obj.result(_W())  # timeout: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/concurrent_futures__base/Future__set_exception__exception_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_concurrent_futures__base_Future__set_exception__exception_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures__base"
# dimension = "type"
# case = "Future__set_exception__exception_as_typed_wrong"
# subject = "concurrent.futures._base.Future.set_exception(exception: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/futures/_base.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.futures._base.Future.set_exception(exception: typed); call it with the wrong type.

typeshed contract: exception is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.futures._base import Future
obj = object.__new__(Future)
try:
    obj.set_exception(_W())  # exception: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/concurrent_futures__base/as_completed__fs_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_concurrent_futures__base_as_completed__fs_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures__base"
# dimension = "type"
# case = "as_completed__fs_as_Iterable_wrong"
# subject = "concurrent.futures._base.as_completed(fs: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/futures/_base.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.futures._base.as_completed(fs: Iterable); call it with the wrong type.

typeshed contract: fs is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.futures._base import as_completed
try:
    as_completed(_W())  # fs: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/concurrent_futures__base/wait__fs_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_concurrent_futures__base_wait__fs_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures__base"
# dimension = "type"
# case = "wait__fs_as_Iterable_wrong"
# subject = "concurrent.futures._base.wait(fs: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/futures/_base.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.futures._base.wait(fs: Iterable); call it with the wrong type.

typeshed contract: fs is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.futures._base import wait
try:
    wait(_W())  # fs: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
