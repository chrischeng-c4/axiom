use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/annotationlib/ForwardRef__init__arg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_annotationlib_ForwardRef__init__arg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "annotationlib"
# dimension = "type"
# case = "ForwardRef__init__arg_as_str_wrong"
# subject = "annotationlib.ForwardRef.__init__(arg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/annotationlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: annotationlib.ForwardRef.__init__(arg: str); call it with the wrong type.

typeshed contract: arg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from annotationlib import ForwardRef
try:
    ForwardRef(12345)  # arg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/annotationlib/annotations_to_string__annotations_as_SupportsItems_wrong.py`.
#[test]
fn test_gen_type_std_libs_annotationlib_annotations_to_string__annotations_as_SupportsItems_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "annotationlib"
# dimension = "type"
# case = "annotations_to_string__annotations_as_SupportsItems_wrong"
# subject = "annotationlib.annotations_to_string(annotations: SupportsItems)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/annotationlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: annotationlib.annotations_to_string(annotations: SupportsItems); call it with the wrong type.

typeshed contract: annotations is SupportsItems. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from annotationlib import annotations_to_string
try:
    annotations_to_string(_W())  # annotations: SupportsItems <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
