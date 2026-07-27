use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/dataclasses/make_dataclass__cls_name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_dataclasses_make_dataclass__cls_name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "type"
# case = "make_dataclass__cls_name_as_str_wrong"
# subject = "dataclasses.make_dataclass(cls_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dataclasses.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: dataclasses.make_dataclass(cls_name: str); call it with the wrong type.

typeshed contract: cls_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from dataclasses import make_dataclass
try:
    make_dataclass(12345, None)  # cls_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
