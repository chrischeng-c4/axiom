use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/zipfile__path/CompleteDirs__resolve_dir__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile__path_CompleteDirs__resolve_dir__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile__path"
# dimension = "type"
# case = "CompleteDirs__resolve_dir__name_as_str_wrong"
# subject = "zipfile._path.CompleteDirs.resolve_dir(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile/_path.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile._path.CompleteDirs.resolve_dir(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipfile._path import CompleteDirs
obj = object.__new__(CompleteDirs)
try:
    obj.resolve_dir(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile__path/Path____truediv____add_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile__path_Path____truediv____add_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile__path"
# dimension = "type"
# case = "Path____truediv____add_as_StrPath_wrong"
# subject = "zipfile._path.Path.__truediv__(add: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile/_path.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile._path.Path.__truediv__(add: StrPath); call it with the wrong type.

typeshed contract: add is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile._path import Path
obj = object.__new__(Path)
try:
    obj.__truediv__(_W())  # add: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile__path/Path__glob__pattern_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile__path_Path__glob__pattern_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile__path"
# dimension = "type"
# case = "Path__glob__pattern_as_str_wrong"
# subject = "zipfile._path.Path.glob(pattern: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile/_path.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile._path.Path.glob(pattern: str); call it with the wrong type.

typeshed contract: pattern is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipfile._path import Path
obj = object.__new__(Path)
try:
    obj.glob(12345)  # pattern: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile__path/Path__init__root_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile__path_Path__init__root_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile__path"
# dimension = "type"
# case = "Path__init__root_as_typed_wrong"
# subject = "zipfile._path.Path.__init__(root: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile/_path.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile._path.Path.__init__(root: typed); call it with the wrong type.

typeshed contract: root is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile._path import Path
try:
    Path(_W())  # root: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile__path/Path__match__path_pattern_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile__path_Path__match__path_pattern_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile__path"
# dimension = "type"
# case = "Path__match__path_pattern_as_str_wrong"
# subject = "zipfile._path.Path.match(path_pattern: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile/_path.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile._path.Path.match(path_pattern: str); call it with the wrong type.

typeshed contract: path_pattern is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipfile._path import Path
obj = object.__new__(Path)
try:
    obj.match(12345)  # path_pattern: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile__path/Path__read_text__encoding_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile__path_Path__read_text__encoding_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile__path"
# dimension = "type"
# case = "Path__read_text__encoding_as_typed_wrong"
# subject = "zipfile._path.Path.read_text(encoding: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile/_path.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile._path.Path.read_text(encoding: typed); call it with the wrong type.

typeshed contract: encoding is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile._path import Path
obj = object.__new__(Path)
try:
    obj.read_text(_W())  # encoding: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile__path/Path__rglob__pattern_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile__path_Path__rglob__pattern_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile__path"
# dimension = "type"
# case = "Path__rglob__pattern_as_str_wrong"
# subject = "zipfile._path.Path.rglob(pattern: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile/_path.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile._path.Path.rglob(pattern: str); call it with the wrong type.

typeshed contract: pattern is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipfile._path import Path
obj = object.__new__(Path)
try:
    obj.rglob(12345)  # pattern: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
