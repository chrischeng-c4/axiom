use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/asyncio_timeouts/Timeout____aexit____exc_type_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_timeouts_Timeout____aexit____exc_type_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_timeouts"
# dimension = "type"
# case = "Timeout____aexit____exc_type_as_typed_wrong"
# subject = "asyncio.timeouts.Timeout.__aexit__(exc_type: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/timeouts.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.timeouts.Timeout.__aexit__(exc_type: typed); call it with the wrong type.

typeshed contract: exc_type is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.timeouts import Timeout
obj = object.__new__(Timeout)
try:
    obj.__aexit__(_W(), None, None)  # exc_type: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_timeouts/Timeout__init__when_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_timeouts_Timeout__init__when_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_timeouts"
# dimension = "type"
# case = "Timeout__init__when_as_typed_wrong"
# subject = "asyncio.timeouts.Timeout.__init__(when: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/timeouts.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.timeouts.Timeout.__init__(when: typed); call it with the wrong type.

typeshed contract: when is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.timeouts import Timeout
try:
    Timeout(_W())  # when: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_timeouts/Timeout__reschedule__when_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_timeouts_Timeout__reschedule__when_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_timeouts"
# dimension = "type"
# case = "Timeout__reschedule__when_as_typed_wrong"
# subject = "asyncio.timeouts.Timeout.reschedule(when: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/timeouts.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.timeouts.Timeout.reschedule(when: typed); call it with the wrong type.

typeshed contract: when is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.timeouts import Timeout
obj = object.__new__(Timeout)
try:
    obj.reschedule(_W())  # when: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_timeouts/timeout__delay_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_timeouts_timeout__delay_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_timeouts"
# dimension = "type"
# case = "timeout__delay_as_typed_wrong"
# subject = "asyncio.timeouts.timeout(delay: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/timeouts.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.timeouts.timeout(delay: typed); call it with the wrong type.

typeshed contract: delay is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.timeouts import timeout
try:
    timeout(_W())  # delay: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_timeouts/timeout_at__when_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_timeouts_timeout_at__when_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_timeouts"
# dimension = "type"
# case = "timeout_at__when_as_typed_wrong"
# subject = "asyncio.timeouts.timeout_at(when: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/timeouts.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.timeouts.timeout_at(when: typed); call it with the wrong type.

typeshed contract: when is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.timeouts import timeout_at
try:
    timeout_at(_W())  # when: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
