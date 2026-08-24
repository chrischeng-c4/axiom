use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/tabnanny/NannyNag__init__lineno_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_tabnanny_NannyNag__init__lineno_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tabnanny"
# dimension = "type"
# case = "NannyNag__init__lineno_as_int_wrong"
# subject = "tabnanny.NannyNag.__init__(lineno: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tabnanny.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tabnanny.NannyNag.__init__(lineno: int); call it with the wrong type.

typeshed contract: lineno is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tabnanny import NannyNag
try:
    NannyNag("not_an_int", "", "")  # lineno: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tabnanny/check__file_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_tabnanny_check__file_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tabnanny"
# dimension = "type"
# case = "check__file_as_StrOrBytesPath_wrong"
# subject = "tabnanny.check(file: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tabnanny.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tabnanny.check(file: StrOrBytesPath); call it with the wrong type.

typeshed contract: file is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tabnanny import check
try:
    check(_W())  # file: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tabnanny/process_tokens__tokens_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_tabnanny_process_tokens__tokens_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tabnanny"
# dimension = "type"
# case = "process_tokens__tokens_as_Iterable_wrong"
# subject = "tabnanny.process_tokens(tokens: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tabnanny.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tabnanny.process_tokens(tokens: Iterable); call it with the wrong type.

typeshed contract: tokens is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tabnanny import process_tokens
try:
    process_tokens(_W())  # tokens: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
