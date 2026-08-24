use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/asyncio_exceptions/IncompleteReadError__init__partial_as_bytes_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_exceptions_IncompleteReadError__init__partial_as_bytes_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_exceptions"
# dimension = "type"
# case = "IncompleteReadError__init__partial_as_bytes_wrong"
# subject = "asyncio.exceptions.IncompleteReadError.__init__(partial: bytes)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/exceptions.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.exceptions.IncompleteReadError.__init__(partial: bytes); call it with the wrong type.

typeshed contract: partial is bytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.exceptions import IncompleteReadError
try:
    IncompleteReadError(12345, None)  # partial: bytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_exceptions/LimitOverrunError__init__message_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_exceptions_LimitOverrunError__init__message_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_exceptions"
# dimension = "type"
# case = "LimitOverrunError__init__message_as_str_wrong"
# subject = "asyncio.exceptions.LimitOverrunError.__init__(message: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/exceptions.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.exceptions.LimitOverrunError.__init__(message: str); call it with the wrong type.

typeshed contract: message is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.exceptions import LimitOverrunError
try:
    LimitOverrunError(12345, 0)  # message: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
