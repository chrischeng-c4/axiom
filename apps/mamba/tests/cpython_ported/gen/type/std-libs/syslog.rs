use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/syslog/LOG_MASK__pri_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_syslog_LOG_MASK__pri_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "syslog"
# dimension = "type"
# case = "LOG_MASK__pri_as_int_wrong"
# subject = "syslog.LOG_MASK(pri: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/syslog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: syslog.LOG_MASK(pri: int); call it with the wrong type.

typeshed contract: pri is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from syslog import LOG_MASK
try:
    LOG_MASK("not_an_int")  # pri: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/syslog/LOG_UPTO__pri_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_syslog_LOG_UPTO__pri_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "syslog"
# dimension = "type"
# case = "LOG_UPTO__pri_as_int_wrong"
# subject = "syslog.LOG_UPTO(pri: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/syslog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: syslog.LOG_UPTO(pri: int); call it with the wrong type.

typeshed contract: pri is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from syslog import LOG_UPTO
try:
    LOG_UPTO("not_an_int")  # pri: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/syslog/openlog__ident_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_syslog_openlog__ident_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "syslog"
# dimension = "type"
# case = "openlog__ident_as_str_wrong"
# subject = "syslog.openlog(ident: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/syslog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: syslog.openlog(ident: str); call it with the wrong type.

typeshed contract: ident is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from syslog import openlog
try:
    openlog(12345)  # ident: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/syslog/setlogmask__maskpri_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_syslog_setlogmask__maskpri_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "syslog"
# dimension = "type"
# case = "setlogmask__maskpri_as_int_wrong"
# subject = "syslog.setlogmask(maskpri: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/syslog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: syslog.setlogmask(maskpri: int); call it with the wrong type.

typeshed contract: maskpri is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from syslog import setlogmask
try:
    setlogmask("not_an_int")  # maskpri: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
