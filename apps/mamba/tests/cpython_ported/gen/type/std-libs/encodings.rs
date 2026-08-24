use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/encodings/normalize_encoding__encoding_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_normalize_encoding__encoding_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings"
# dimension = "type"
# case = "normalize_encoding__encoding_as_typed_wrong"
# subject = "encodings.normalize_encoding(encoding: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.normalize_encoding(encoding: typed); call it with the wrong type.

typeshed contract: encoding is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from encodings import normalize_encoding
try:
    normalize_encoding(_W())  # encoding: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/encodings/search_function__encoding_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_search_function__encoding_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings"
# dimension = "type"
# case = "search_function__encoding_as_str_wrong"
# subject = "encodings.search_function(encoding: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.search_function(encoding: str); call it with the wrong type.

typeshed contract: encoding is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings import search_function
try:
    search_function(12345)  # encoding: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/encodings/win32_code_page_search_function__encoding_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_win32_code_page_search_function__encoding_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings"
# dimension = "type"
# case = "win32_code_page_search_function__encoding_as_str_wrong"
# subject = "encodings.win32_code_page_search_function(encoding: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.win32_code_page_search_function(encoding: str); call it with the wrong type.

typeshed contract: encoding is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings import win32_code_page_search_function
try:
    win32_code_page_search_function(12345)  # encoding: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
