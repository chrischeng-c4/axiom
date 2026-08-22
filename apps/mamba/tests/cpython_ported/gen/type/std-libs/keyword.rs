use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/keyword/iskeyword__s_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_keyword_iskeyword__s_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "type"
# case = "iskeyword__s_as_str_wrong"
# subject = "keyword.iskeyword(s: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/keyword.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: keyword.iskeyword(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from keyword import iskeyword
try:
    iskeyword(12345)  # s: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/keyword/issoftkeyword__s_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_keyword_issoftkeyword__s_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "type"
# case = "issoftkeyword__s_as_str_wrong"
# subject = "keyword.issoftkeyword(s: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/keyword.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: keyword.issoftkeyword(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from keyword import issoftkeyword
try:
    issoftkeyword(12345)  # s: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
