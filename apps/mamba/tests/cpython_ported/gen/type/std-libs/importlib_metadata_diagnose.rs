use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/importlib_metadata_diagnose/inspect__path_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_metadata_diagnose_inspect__path_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_metadata_diagnose"
# dimension = "type"
# case = "inspect__path_as_str_wrong"
# subject = "importlib.metadata.diagnose.inspect(path: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/metadata/diagnose.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.metadata.diagnose.inspect(path: str); call it with the wrong type.

typeshed contract: path is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from importlib.metadata.diagnose import inspect
try:
    inspect(12345)  # path: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
