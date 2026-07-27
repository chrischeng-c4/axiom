use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/asyncio_queues/Queue__init__maxsize_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_queues_Queue__init__maxsize_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_queues"
# dimension = "type"
# case = "Queue__init__maxsize_as_int_wrong"
# subject = "asyncio.queues.Queue.__init__(maxsize: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/queues.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.queues.Queue.__init__(maxsize: int); call it with the wrong type.

typeshed contract: maxsize is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.queues import Queue
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
