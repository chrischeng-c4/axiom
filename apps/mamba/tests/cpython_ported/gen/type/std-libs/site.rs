use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/site/addpackage__sitedir_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_site_addpackage__sitedir_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "site"
# dimension = "type"
# case = "addpackage__sitedir_as_StrPath_wrong"
# subject = "site.addpackage(sitedir: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/site.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: site.addpackage(sitedir: StrPath); call it with the wrong type.

typeshed contract: sitedir is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from site import addpackage
try:
    addpackage(_W(), None, None)  # sitedir: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/site/addsitedir__sitedir_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_site_addsitedir__sitedir_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "site"
# dimension = "type"
# case = "addsitedir__sitedir_as_str_wrong"
# subject = "site.addsitedir(sitedir: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/site.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: site.addsitedir(sitedir: str); call it with the wrong type.

typeshed contract: sitedir is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from site import addsitedir
try:
    addsitedir(12345)  # sitedir: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/site/getsitepackages__prefixes_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_site_getsitepackages__prefixes_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "site"
# dimension = "type"
# case = "getsitepackages__prefixes_as_typed_wrong"
# subject = "site.getsitepackages(prefixes: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/site.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: site.getsitepackages(prefixes: typed); call it with the wrong type.

typeshed contract: prefixes is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from site import getsitepackages
try:
    getsitepackages(_W())  # prefixes: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
