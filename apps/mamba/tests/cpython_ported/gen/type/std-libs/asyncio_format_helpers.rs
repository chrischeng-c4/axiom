use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/asyncio_format_helpers/extract_stack__f_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_format_helpers_extract_stack__f_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_format_helpers"
# dimension = "type"
# case = "extract_stack__f_as_typed_wrong"
# subject = "asyncio.format_helpers.extract_stack(f: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/format_helpers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.format_helpers.extract_stack(f: typed); call it with the wrong type.

typeshed contract: f is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.format_helpers import extract_stack
try:
    extract_stack(_W())  # f: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
