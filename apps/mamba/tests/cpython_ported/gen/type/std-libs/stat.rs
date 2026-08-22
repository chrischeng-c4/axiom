use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_stat/S_IFMT__mode_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__stat_S_IFMT__mode_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_stat"
# dimension = "type"
# case = "S_IFMT__mode_as_int_wrong"
# subject = "_stat.S_IFMT(mode: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_stat.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _stat.S_IFMT(mode: int); call it with the wrong type.

typeshed contract: mode is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _stat import S_IFMT
try:
    S_IFMT("not_an_int")  # mode: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_stat/S_IMODE__mode_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__stat_S_IMODE__mode_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_stat"
# dimension = "type"
# case = "S_IMODE__mode_as_int_wrong"
# subject = "_stat.S_IMODE(mode: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_stat.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _stat.S_IMODE(mode: int); call it with the wrong type.

typeshed contract: mode is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _stat import S_IMODE
try:
    S_IMODE("not_an_int")  # mode: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_stat/S_ISBLK__mode_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__stat_S_ISBLK__mode_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_stat"
# dimension = "type"
# case = "S_ISBLK__mode_as_int_wrong"
# subject = "_stat.S_ISBLK(mode: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_stat.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _stat.S_ISBLK(mode: int); call it with the wrong type.

typeshed contract: mode is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _stat import S_ISBLK
try:
    S_ISBLK("not_an_int")  # mode: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_stat/S_ISCHR__mode_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__stat_S_ISCHR__mode_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_stat"
# dimension = "type"
# case = "S_ISCHR__mode_as_int_wrong"
# subject = "_stat.S_ISCHR(mode: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_stat.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _stat.S_ISCHR(mode: int); call it with the wrong type.

typeshed contract: mode is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _stat import S_ISCHR
try:
    S_ISCHR("not_an_int")  # mode: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_stat/S_ISDIR__mode_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__stat_S_ISDIR__mode_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_stat"
# dimension = "type"
# case = "S_ISDIR__mode_as_int_wrong"
# subject = "_stat.S_ISDIR(mode: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_stat.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _stat.S_ISDIR(mode: int); call it with the wrong type.

typeshed contract: mode is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _stat import S_ISDIR
try:
    S_ISDIR("not_an_int")  # mode: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_stat/S_ISDOOR__mode_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__stat_S_ISDOOR__mode_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_stat"
# dimension = "type"
# case = "S_ISDOOR__mode_as_int_wrong"
# subject = "_stat.S_ISDOOR(mode: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_stat.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _stat.S_ISDOOR(mode: int); call it with the wrong type.

typeshed contract: mode is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _stat import S_ISDOOR
try:
    S_ISDOOR("not_an_int")  # mode: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_stat/S_ISFIFO__mode_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__stat_S_ISFIFO__mode_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_stat"
# dimension = "type"
# case = "S_ISFIFO__mode_as_int_wrong"
# subject = "_stat.S_ISFIFO(mode: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_stat.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _stat.S_ISFIFO(mode: int); call it with the wrong type.

typeshed contract: mode is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _stat import S_ISFIFO
try:
    S_ISFIFO("not_an_int")  # mode: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_stat/S_ISLNK__mode_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__stat_S_ISLNK__mode_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_stat"
# dimension = "type"
# case = "S_ISLNK__mode_as_int_wrong"
# subject = "_stat.S_ISLNK(mode: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_stat.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _stat.S_ISLNK(mode: int); call it with the wrong type.

typeshed contract: mode is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _stat import S_ISLNK
try:
    S_ISLNK("not_an_int")  # mode: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_stat/S_ISPORT__mode_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__stat_S_ISPORT__mode_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_stat"
# dimension = "type"
# case = "S_ISPORT__mode_as_int_wrong"
# subject = "_stat.S_ISPORT(mode: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_stat.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _stat.S_ISPORT(mode: int); call it with the wrong type.

typeshed contract: mode is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _stat import S_ISPORT
try:
    S_ISPORT("not_an_int")  # mode: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_stat/S_ISREG__mode_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__stat_S_ISREG__mode_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_stat"
# dimension = "type"
# case = "S_ISREG__mode_as_int_wrong"
# subject = "_stat.S_ISREG(mode: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_stat.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _stat.S_ISREG(mode: int); call it with the wrong type.

typeshed contract: mode is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _stat import S_ISREG
try:
    S_ISREG("not_an_int")  # mode: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_stat/S_ISSOCK__mode_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__stat_S_ISSOCK__mode_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_stat"
# dimension = "type"
# case = "S_ISSOCK__mode_as_int_wrong"
# subject = "_stat.S_ISSOCK(mode: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_stat.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _stat.S_ISSOCK(mode: int); call it with the wrong type.

typeshed contract: mode is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _stat import S_ISSOCK
try:
    S_ISSOCK("not_an_int")  # mode: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_stat/S_ISWHT__mode_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__stat_S_ISWHT__mode_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_stat"
# dimension = "type"
# case = "S_ISWHT__mode_as_int_wrong"
# subject = "_stat.S_ISWHT(mode: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_stat.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _stat.S_ISWHT(mode: int); call it with the wrong type.

typeshed contract: mode is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _stat import S_ISWHT
try:
    S_ISWHT("not_an_int")  # mode: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_stat/filemode__mode_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__stat_filemode__mode_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_stat"
# dimension = "type"
# case = "filemode__mode_as_int_wrong"
# subject = "_stat.filemode(mode: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_stat.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _stat.filemode(mode: int); call it with the wrong type.

typeshed contract: mode is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _stat import filemode
try:
    filemode("not_an_int")  # mode: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
