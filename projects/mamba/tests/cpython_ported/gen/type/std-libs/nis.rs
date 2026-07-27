use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/nis/cat__map_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_nis_cat__map_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "nis"
# dimension = "type"
# case = "cat__map_as_str_wrong"
# subject = "nis.cat(map: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/nis.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: nis.cat(map: str); call it with the wrong type.

typeshed contract: map is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from nis import cat
try:
    cat(12345)  # map: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/nis/maps__domain_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_nis_maps__domain_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "nis"
# dimension = "type"
# case = "maps__domain_as_str_wrong"
# subject = "nis.maps(domain: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/nis.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: nis.maps(domain: str); call it with the wrong type.

typeshed contract: domain is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from nis import maps
try:
    maps(12345)  # domain: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/nis/match__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_nis_match__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "nis"
# dimension = "type"
# case = "match__key_as_str_wrong"
# subject = "nis.match(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/nis.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: nis.match(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from nis import match
try:
    match(12345, "")  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
