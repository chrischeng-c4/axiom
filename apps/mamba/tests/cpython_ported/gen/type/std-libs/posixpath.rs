use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/posixpath/abspath__path_as_AnyStr_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_abspath__path_as_AnyStr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "abspath__path_as_AnyStr_wrong"
# subject = "posixpath.abspath(path: AnyStr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.abspath(path: AnyStr); call it with the wrong type.

typeshed contract: path is AnyStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import abspath
try:
    abspath(_W())  # path: AnyStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/abspath__path_as_PathLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_abspath__path_as_PathLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "abspath__path_as_PathLike_wrong"
# subject = "posixpath.abspath(path: PathLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.abspath(path: PathLike); call it with the wrong type.

typeshed contract: path is PathLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import abspath
try:
    abspath(_W())  # path: PathLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/basename__p_as_AnyOrLiteralStr_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_basename__p_as_AnyOrLiteralStr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "basename__p_as_AnyOrLiteralStr_wrong"
# subject = "posixpath.basename(p: AnyOrLiteralStr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.basename(p: AnyOrLiteralStr); call it with the wrong type.

typeshed contract: p is AnyOrLiteralStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import basename
try:
    basename(_W())  # p: AnyOrLiteralStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/basename__p_as_PathLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_basename__p_as_PathLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "basename__p_as_PathLike_wrong"
# subject = "posixpath.basename(p: PathLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.basename(p: PathLike); call it with the wrong type.

typeshed contract: p is PathLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import basename
try:
    basename(_W())  # p: PathLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/commonpath__paths_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_commonpath__paths_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "commonpath__paths_as_Iterable_wrong"
# subject = "posixpath.commonpath(paths: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.commonpath(paths: Iterable); call it with the wrong type.

typeshed contract: paths is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce).

This case pins the literal-container element gap: the outer list is iterable,
but its bare user-class element is still wrong-typed for the path contract."""

class _W:
    pass


from posixpath import commonpath
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

/// Ported from `tests/cpython/type/std-libs/posixpath/dirname__p_as_AnyOrLiteralStr_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_dirname__p_as_AnyOrLiteralStr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "dirname__p_as_AnyOrLiteralStr_wrong"
# subject = "posixpath.dirname(p: AnyOrLiteralStr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.dirname(p: AnyOrLiteralStr); call it with the wrong type.

typeshed contract: p is AnyOrLiteralStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import dirname
try:
    dirname(_W())  # p: AnyOrLiteralStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/dirname__p_as_PathLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_dirname__p_as_PathLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "dirname__p_as_PathLike_wrong"
# subject = "posixpath.dirname(p: PathLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.dirname(p: PathLike); call it with the wrong type.

typeshed contract: p is PathLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import dirname
try:
    dirname(_W())  # p: PathLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/expanduser__path_as_AnyStr_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_expanduser__path_as_AnyStr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "expanduser__path_as_AnyStr_wrong"
# subject = "posixpath.expanduser(path: AnyStr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.expanduser(path: AnyStr); call it with the wrong type.

typeshed contract: path is AnyStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import expanduser
try:
    expanduser(_W())  # path: AnyStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/expanduser__path_as_PathLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_expanduser__path_as_PathLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "expanduser__path_as_PathLike_wrong"
# subject = "posixpath.expanduser(path: PathLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.expanduser(path: PathLike); call it with the wrong type.

typeshed contract: path is PathLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import expanduser
try:
    expanduser(_W())  # path: PathLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/expandvars__path_as_AnyStr_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_expandvars__path_as_AnyStr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "expandvars__path_as_AnyStr_wrong"
# subject = "posixpath.expandvars(path: AnyStr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.expandvars(path: AnyStr); call it with the wrong type.

typeshed contract: path is AnyStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import expandvars
try:
    expandvars(_W())  # path: AnyStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/expandvars__path_as_PathLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_expandvars__path_as_PathLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "expandvars__path_as_PathLike_wrong"
# subject = "posixpath.expandvars(path: PathLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.expandvars(path: PathLike); call it with the wrong type.

typeshed contract: path is PathLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import expandvars
try:
    expandvars(_W())  # path: PathLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/isabs__s_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_isabs__s_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "isabs__s_as_StrOrBytesPath_wrong"
# subject = "posixpath.isabs(s: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.isabs(s: StrOrBytesPath); call it with the wrong type.

typeshed contract: s is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import isabs
try:
    isabs(_W())  # s: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/isjunction__path_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_isjunction__path_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "isjunction__path_as_StrOrBytesPath_wrong"
# subject = "posixpath.isjunction(path: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.isjunction(path: StrOrBytesPath); call it with the wrong type.

typeshed contract: path is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import isjunction
try:
    isjunction(_W())  # path: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/islink__path_as_FileDescriptorOrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_islink__path_as_FileDescriptorOrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "islink__path_as_FileDescriptorOrPath_wrong"
# subject = "posixpath.islink(path: FileDescriptorOrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.islink(path: FileDescriptorOrPath); call it with the wrong type.

typeshed contract: path is FileDescriptorOrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import islink
try:
    islink(_W())  # path: FileDescriptorOrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/ismount__path_as_FileDescriptorOrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_ismount__path_as_FileDescriptorOrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "ismount__path_as_FileDescriptorOrPath_wrong"
# subject = "posixpath.ismount(path: FileDescriptorOrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.ismount(path: FileDescriptorOrPath); call it with the wrong type.

typeshed contract: path is FileDescriptorOrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import ismount
try:
    ismount(_W())  # path: FileDescriptorOrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/join__a_as_BytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_join__a_as_BytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "join__a_as_BytesPath_wrong"
# subject = "posixpath.join(a: BytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.join(a: BytesPath); call it with the wrong type.

typeshed contract: a is BytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import join
try:
    join(_W())  # a: BytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/join__a_as_LiteralString_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_join__a_as_LiteralString_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "join__a_as_LiteralString_wrong"
# subject = "posixpath.join(a: LiteralString)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.join(a: LiteralString); call it with the wrong type.

typeshed contract: a is LiteralString. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import join
try:
    join(_W())  # a: LiteralString <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/join__a_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_join__a_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "join__a_as_StrPath_wrong"
# subject = "posixpath.join(a: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.join(a: StrPath); call it with the wrong type.

typeshed contract: a is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import join
try:
    join(_W())  # a: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/lexists__path_as_FileDescriptorOrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_lexists__path_as_FileDescriptorOrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "lexists__path_as_FileDescriptorOrPath_wrong"
# subject = "posixpath.lexists(path: FileDescriptorOrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.lexists(path: FileDescriptorOrPath); call it with the wrong type.

typeshed contract: path is FileDescriptorOrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import lexists
try:
    lexists(_W())  # path: FileDescriptorOrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/normcase__s_as_AnyOrLiteralStr_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_normcase__s_as_AnyOrLiteralStr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "normcase__s_as_AnyOrLiteralStr_wrong"
# subject = "posixpath.normcase(s: AnyOrLiteralStr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.normcase(s: AnyOrLiteralStr); call it with the wrong type.

typeshed contract: s is AnyOrLiteralStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import normcase
try:
    normcase(_W())  # s: AnyOrLiteralStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/normcase__s_as_PathLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_normcase__s_as_PathLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "normcase__s_as_PathLike_wrong"
# subject = "posixpath.normcase(s: PathLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.normcase(s: PathLike); call it with the wrong type.

typeshed contract: s is PathLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import normcase
try:
    normcase(_W())  # s: PathLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/normpath__path_as_AnyOrLiteralStr_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_normpath__path_as_AnyOrLiteralStr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "normpath__path_as_AnyOrLiteralStr_wrong"
# subject = "posixpath.normpath(path: AnyOrLiteralStr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.normpath(path: AnyOrLiteralStr); call it with the wrong type.

typeshed contract: path is AnyOrLiteralStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import normpath
try:
    normpath(_W())  # path: AnyOrLiteralStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/normpath__path_as_PathLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_normpath__path_as_PathLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "normpath__path_as_PathLike_wrong"
# subject = "posixpath.normpath(path: PathLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.normpath(path: PathLike); call it with the wrong type.

typeshed contract: path is PathLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import normpath
try:
    normpath(_W())  # path: PathLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/realpath__filename_as_AnyStr_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_realpath__filename_as_AnyStr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "realpath__filename_as_AnyStr_wrong"
# subject = "posixpath.realpath(filename: AnyStr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.realpath(filename: AnyStr); call it with the wrong type.

typeshed contract: filename is AnyStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import realpath
try:
    realpath(_W())  # filename: AnyStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/realpath__filename_as_PathLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_realpath__filename_as_PathLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "realpath__filename_as_PathLike_wrong"
# subject = "posixpath.realpath(filename: PathLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.realpath(filename: PathLike); call it with the wrong type.

typeshed contract: filename is PathLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import realpath
try:
    realpath(_W())  # filename: PathLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/relpath__path_as_BytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_relpath__path_as_BytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "relpath__path_as_BytesPath_wrong"
# subject = "posixpath.relpath(path: BytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.relpath(path: BytesPath); call it with the wrong type.

typeshed contract: path is BytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import relpath
try:
    relpath(_W())  # path: BytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/relpath__path_as_LiteralString_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_relpath__path_as_LiteralString_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "relpath__path_as_LiteralString_wrong"
# subject = "posixpath.relpath(path: LiteralString)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.relpath(path: LiteralString); call it with the wrong type.

typeshed contract: path is LiteralString. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import relpath
try:
    relpath(_W())  # path: LiteralString <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/relpath__path_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_relpath__path_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "relpath__path_as_StrPath_wrong"
# subject = "posixpath.relpath(path: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.relpath(path: StrPath); call it with the wrong type.

typeshed contract: path is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import relpath
try:
    relpath(_W())  # path: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/split__p_as_AnyOrLiteralStr_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_split__p_as_AnyOrLiteralStr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "split__p_as_AnyOrLiteralStr_wrong"
# subject = "posixpath.split(p: AnyOrLiteralStr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.split(p: AnyOrLiteralStr); call it with the wrong type.

typeshed contract: p is AnyOrLiteralStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import split
try:
    split(_W())  # p: AnyOrLiteralStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/split__p_as_PathLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_split__p_as_PathLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "split__p_as_PathLike_wrong"
# subject = "posixpath.split(p: PathLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.split(p: PathLike); call it with the wrong type.

typeshed contract: p is PathLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import split
try:
    split(_W())  # p: PathLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/splitdrive__p_as_AnyOrLiteralStr_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_splitdrive__p_as_AnyOrLiteralStr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "splitdrive__p_as_AnyOrLiteralStr_wrong"
# subject = "posixpath.splitdrive(p: AnyOrLiteralStr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.splitdrive(p: AnyOrLiteralStr); call it with the wrong type.

typeshed contract: p is AnyOrLiteralStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import splitdrive
try:
    splitdrive(_W())  # p: AnyOrLiteralStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/splitdrive__p_as_PathLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_splitdrive__p_as_PathLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "splitdrive__p_as_PathLike_wrong"
# subject = "posixpath.splitdrive(p: PathLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.splitdrive(p: PathLike); call it with the wrong type.

typeshed contract: p is PathLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import splitdrive
try:
    splitdrive(_W())  # p: PathLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/splitext__p_as_AnyOrLiteralStr_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_splitext__p_as_AnyOrLiteralStr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "splitext__p_as_AnyOrLiteralStr_wrong"
# subject = "posixpath.splitext(p: AnyOrLiteralStr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.splitext(p: AnyOrLiteralStr); call it with the wrong type.

typeshed contract: p is AnyOrLiteralStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import splitext
try:
    splitext(_W())  # p: AnyOrLiteralStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/splitext__p_as_PathLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_splitext__p_as_PathLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "splitext__p_as_PathLike_wrong"
# subject = "posixpath.splitext(p: PathLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.splitext(p: PathLike); call it with the wrong type.

typeshed contract: p is PathLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import splitext
try:
    splitext(_W())  # p: PathLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/splitroot__p_as_AnyOrLiteralStr_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_splitroot__p_as_AnyOrLiteralStr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "splitroot__p_as_AnyOrLiteralStr_wrong"
# subject = "posixpath.splitroot(p: AnyOrLiteralStr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.splitroot(p: AnyOrLiteralStr); call it with the wrong type.

typeshed contract: p is AnyOrLiteralStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import splitroot
try:
    splitroot(_W())  # p: AnyOrLiteralStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/posixpath/splitroot__p_as_PathLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_posixpath_splitroot__p_as_PathLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "splitroot__p_as_PathLike_wrong"
# subject = "posixpath.splitroot(p: PathLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.splitroot(p: PathLike); call it with the wrong type.

typeshed contract: p is PathLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import splitroot
try:
    splitroot(_W())  # p: PathLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
