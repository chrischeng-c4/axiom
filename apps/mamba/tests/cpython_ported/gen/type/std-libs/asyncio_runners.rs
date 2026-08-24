use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/asyncio_runners/run__main_as_Coroutine_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_runners_run__main_as_Coroutine_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_runners"
# dimension = "type"
# case = "run__main_as_Coroutine_wrong"
# subject = "asyncio.runners.run(main: Coroutine)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/runners.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.runners.run(main: Coroutine); call it with the wrong type.

typeshed contract: main is Coroutine. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.runners import run
try:
    run(_W())  # main: Coroutine <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
