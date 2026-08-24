use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/ntpath/commonpath__paths_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_ntpath_commonpath__paths_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ntpath"
# dimension = "type"
# case = "commonpath__paths_as_Iterable_wrong"
# subject = "ntpath.commonpath(paths: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ntpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ntpath.commonpath(paths: Iterable); call it with the wrong type.

typeshed contract: paths is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce).

This case pins the literal-container element gap: the outer list is iterable,
but its bare user-class element is still wrong-typed for the path contract."""

class _W:
    pass


from ntpath import commonpath
try:
    commonpath([_W()])  # paths: Iterable <- literal container with wrong-typed element
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ntpath/isreserved__path_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_ntpath_isreserved__path_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ntpath"
# dimension = "type"
# case = "isreserved__path_as_StrOrBytesPath_wrong"
# subject = "ntpath.isreserved(path: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ntpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ntpath.isreserved(path: StrOrBytesPath); call it with the wrong type.

typeshed contract: path is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ntpath import isreserved
try:
    isreserved(_W())  # path: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ntpath/join__path_as_BytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_ntpath_join__path_as_BytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ntpath"
# dimension = "type"
# case = "join__path_as_BytesPath_wrong"
# subject = "ntpath.join(path: BytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ntpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ntpath.join(path: BytesPath); call it with the wrong type.

typeshed contract: path is BytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ntpath import join
try:
    join(_W())  # path: BytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ntpath/join__path_as_LiteralString_wrong.py`.
#[test]
fn test_gen_type_std_libs_ntpath_join__path_as_LiteralString_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ntpath"
# dimension = "type"
# case = "join__path_as_LiteralString_wrong"
# subject = "ntpath.join(path: LiteralString)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ntpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ntpath.join(path: LiteralString); call it with the wrong type.

typeshed contract: path is LiteralString. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ntpath import join
try:
    join(_W())  # path: LiteralString <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ntpath/join__path_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_ntpath_join__path_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ntpath"
# dimension = "type"
# case = "join__path_as_StrPath_wrong"
# subject = "ntpath.join(path: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ntpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ntpath.join(path: StrPath); call it with the wrong type.

typeshed contract: path is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ntpath import join
try:
    join(_W())  # path: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ntpath/realpath__path_as_AnyStr_wrong.py`.
#[test]
fn test_gen_type_std_libs_ntpath_realpath__path_as_AnyStr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ntpath"
# dimension = "type"
# case = "realpath__path_as_AnyStr_wrong"
# subject = "ntpath.realpath(path: AnyStr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ntpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ntpath.realpath(path: AnyStr); call it with the wrong type.

typeshed contract: path is AnyStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ntpath import realpath
try:
    realpath(_W())  # path: AnyStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ntpath/realpath__path_as_PathLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_ntpath_realpath__path_as_PathLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ntpath"
# dimension = "type"
# case = "realpath__path_as_PathLike_wrong"
# subject = "ntpath.realpath(path: PathLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ntpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ntpath.realpath(path: PathLike); call it with the wrong type.

typeshed contract: path is PathLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ntpath import realpath
try:
    realpath(_W())  # path: PathLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
