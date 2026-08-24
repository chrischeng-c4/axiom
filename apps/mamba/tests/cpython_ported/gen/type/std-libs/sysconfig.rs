use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/sysconfig/get_path__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_sysconfig_get_path__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sysconfig"
# dimension = "type"
# case = "get_path__name_as_str_wrong"
# subject = "sysconfig.get_path(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sysconfig.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sysconfig.get_path(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sysconfig import get_path
try:
    get_path(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/sysconfig/get_paths__scheme_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_sysconfig_get_paths__scheme_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sysconfig"
# dimension = "type"
# case = "get_paths__scheme_as_str_wrong"
# subject = "sysconfig.get_paths(scheme: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sysconfig.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sysconfig.get_paths(scheme: str); call it with the wrong type.

typeshed contract: scheme is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sysconfig import get_paths
try:
    get_paths(12345)  # scheme: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/sysconfig/parse_config_h__fp_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_sysconfig_parse_config_h__fp_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sysconfig"
# dimension = "type"
# case = "parse_config_h__fp_as_IO_wrong"
# subject = "sysconfig.parse_config_h(fp: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sysconfig.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sysconfig.parse_config_h(fp: IO); call it with the wrong type.

typeshed contract: fp is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from sysconfig import parse_config_h
try:
    parse_config_h(_W())  # fp: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
