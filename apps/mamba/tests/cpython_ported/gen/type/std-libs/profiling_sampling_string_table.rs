use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/profiling_sampling_string_table/StringTable__get_string__index_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_profiling_sampling_string_table_StringTable__get_string__index_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profiling_sampling_string_table"
# dimension = "type"
# case = "StringTable__get_string__index_as_int_wrong"
# subject = "profiling.sampling.string_table.StringTable.get_string(index: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profiling/sampling/string_table.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: profiling.sampling.string_table.StringTable.get_string(index: int); call it with the wrong type.

typeshed contract: index is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from profiling.sampling.string_table import StringTable
obj = object.__new__(StringTable)
try:
    obj.get_string("not_an_int")  # index: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
