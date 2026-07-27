use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_dbm/open__filename_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs__dbm_open__filename_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_dbm"
# dimension = "type"
# case = "open__filename_as_StrOrBytesPath_wrong"
# subject = "_dbm.open(filename: StrOrBytesPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_dbm.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _dbm.open(filename: StrOrBytesPath); call it with the wrong type.

typeshed contract: filename is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _dbm import open
try:
    open(_W())  # filename: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_dbm/open__filename_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__dbm_open__filename_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_dbm"
# dimension = "type"
# case = "open__filename_as_str_wrong"
# subject = "_dbm.open(filename: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_dbm.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _dbm.open(filename: str); call it with the wrong type.

typeshed contract: filename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _dbm import open
try:
    open(12345)  # filename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/dbm/open__file_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_dbm_open__file_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "type"
# case = "open__file_as_StrOrBytesPath_wrong"
# subject = "dbm.open(file: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dbm.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: dbm.open(file: StrOrBytesPath); call it with the wrong type.

typeshed contract: file is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from dbm import open
try:
    open(_W())  # file: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/dbm/open__file_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_dbm_open__file_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "type"
# case = "open__file_as_str_wrong"
# subject = "dbm.open(file: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dbm.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: dbm.open(file: str); call it with the wrong type.

typeshed contract: file is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from dbm import open
try:
    open(12345)  # file: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/dbm/whichdb__filename_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_dbm_whichdb__filename_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "type"
# case = "whichdb__filename_as_StrOrBytesPath_wrong"
# subject = "dbm.whichdb(filename: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dbm.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: dbm.whichdb(filename: StrOrBytesPath); call it with the wrong type.

typeshed contract: filename is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from dbm import whichdb
try:
    whichdb(_W())  # filename: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/dbm/whichdb__filename_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_dbm_whichdb__filename_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "type"
# case = "whichdb__filename_as_str_wrong"
# subject = "dbm.whichdb(filename: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dbm.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: dbm.whichdb(filename: str); call it with the wrong type.

typeshed contract: filename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from dbm import whichdb
try:
    whichdb(12345)  # filename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
