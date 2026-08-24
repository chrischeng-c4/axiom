use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/filecmp/cmp__f1_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_filecmp_cmp__f1_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "filecmp"
# dimension = "type"
# case = "cmp__f1_as_StrOrBytesPath_wrong"
# subject = "filecmp.cmp(f1: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/filecmp.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: filecmp.cmp(f1: StrOrBytesPath); call it with the wrong type.

typeshed contract: f1 is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from filecmp import cmp
try:
    cmp(_W(), None)  # f1: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/filecmp/cmpfiles__a_as_GenericPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_filecmp_cmpfiles__a_as_GenericPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "filecmp"
# dimension = "type"
# case = "cmpfiles__a_as_GenericPath_wrong"
# subject = "filecmp.cmpfiles(a: GenericPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/filecmp.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: filecmp.cmpfiles(a: GenericPath); call it with the wrong type.

typeshed contract: a is GenericPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from filecmp import cmpfiles
try:
    cmpfiles(_W(), None, None)  # a: GenericPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/filecmp/dircmp__init__a_as_GenericPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_filecmp_dircmp__init__a_as_GenericPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "filecmp"
# dimension = "type"
# case = "dircmp__init__a_as_GenericPath_wrong"
# subject = "filecmp.dircmp.__init__(a: GenericPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/filecmp.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: filecmp.dircmp.__init__(a: GenericPath); call it with the wrong type.

typeshed contract: a is GenericPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from filecmp import dircmp
try:
    dircmp(_W(), None)  # a: GenericPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
