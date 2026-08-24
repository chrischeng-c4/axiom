use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/asyncio_coroutines/coroutine__func_as__FunctionT_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_coroutines_coroutine__func_as__FunctionT_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_coroutines"
# dimension = "type"
# case = "coroutine__func_as__FunctionT_wrong"
# subject = "asyncio.coroutines.coroutine(func: _FunctionT)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/coroutines.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.coroutines.coroutine(func: _FunctionT); call it with the wrong type.

typeshed contract: func is _FunctionT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.coroutines import coroutine
try:
    coroutine(_W())  # func: _FunctionT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
