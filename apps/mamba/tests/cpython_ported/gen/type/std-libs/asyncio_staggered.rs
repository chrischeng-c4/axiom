use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/asyncio_staggered/staggered_race__coro_fns_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_staggered_staggered_race__coro_fns_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_staggered"
# dimension = "type"
# case = "staggered_race__coro_fns_as_Iterable_wrong"
# subject = "asyncio.staggered.staggered_race(coro_fns: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/staggered.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.staggered.staggered_race(coro_fns: Iterable); call it with the wrong type.

typeshed contract: coro_fns is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.staggered import staggered_race
try:
    staggered_race(_W(), None)  # coro_fns: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
