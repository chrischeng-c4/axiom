use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/pathlib/Path____exit____t_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path____exit____t_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path____exit____t_as_typed_wrong"
# subject = "pathlib.Path.__exit__(t: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.__exit__(t: typed); call it with the wrong type.

typeshed contract: t is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import Path
obj = object.__new__(Path)
try:
    obj.__exit__(_W(), None, None)  # t: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__chmod__mode_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__chmod__mode_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__chmod__mode_as_int_wrong"
# subject = "pathlib.Path.chmod(mode: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.chmod(mode: int); call it with the wrong type.

typeshed contract: mode is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pathlib import Path
obj = object.__new__(Path)
try:
    obj.chmod("not_an_int")  # mode: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__copy__target_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__copy__target_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__copy__target_as_StrPath_wrong"
# subject = "pathlib.Path.copy(target: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.copy(target: StrPath); call it with the wrong type.

typeshed contract: target is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import Path
obj = object.__new__(Path)
try:
    obj.copy(_W())  # target: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__copy__target_as__PathT_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__copy__target_as__PathT_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__copy__target_as__PathT_wrong"
# subject = "pathlib.Path.copy(target: _PathT)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.copy(target: _PathT); call it with the wrong type.

typeshed contract: target is _PathT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import Path
obj = object.__new__(Path)
try:
    obj.copy(_W())  # target: _PathT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__copy_into__target_dir_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__copy_into__target_dir_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__copy_into__target_dir_as_StrPath_wrong"
# subject = "pathlib.Path.copy_into(target_dir: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.copy_into(target_dir: StrPath); call it with the wrong type.

typeshed contract: target_dir is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import Path
obj = object.__new__(Path)
try:
    obj.copy_into(_W())  # target_dir: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__copy_into__target_dir_as__PathT_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__copy_into__target_dir_as__PathT_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__copy_into__target_dir_as__PathT_wrong"
# subject = "pathlib.Path.copy_into(target_dir: _PathT)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.copy_into(target_dir: _PathT); call it with the wrong type.

typeshed contract: target_dir is _PathT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import Path
obj = object.__new__(Path)
try:
    obj.copy_into(_W())  # target_dir: _PathT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__from_uri__uri_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__from_uri__uri_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__from_uri__uri_as_str_wrong"
# subject = "pathlib.Path.from_uri(uri: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.from_uri(uri: str); call it with the wrong type.

typeshed contract: uri is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pathlib import Path
try:
    Path.from_uri(12345)  # uri: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__glob__pattern_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__glob__pattern_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__glob__pattern_as_str_wrong"
# subject = "pathlib.Path.glob(pattern: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.glob(pattern: str); call it with the wrong type.

typeshed contract: pattern is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pathlib import Path
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

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__hardlink_to__target_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__hardlink_to__target_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__hardlink_to__target_as_StrOrBytesPath_wrong"
# subject = "pathlib.Path.hardlink_to(target: StrOrBytesPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.hardlink_to(target: StrOrBytesPath); call it with the wrong type.

typeshed contract: target is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import Path
obj = object.__new__(Path)
try:
    obj.hardlink_to(_W())  # target: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__lchmod__mode_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__lchmod__mode_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__lchmod__mode_as_int_wrong"
# subject = "pathlib.Path.lchmod(mode: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.lchmod(mode: int); call it with the wrong type.

typeshed contract: mode is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pathlib import Path
obj = object.__new__(Path)
try:
    obj.lchmod("not_an_int")  # mode: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__link_to__target_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__link_to__target_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__link_to__target_as_StrOrBytesPath_wrong"
# subject = "pathlib.Path.link_to(target: StrOrBytesPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.link_to(target: StrOrBytesPath); call it with the wrong type.

typeshed contract: target is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import Path
obj = object.__new__(Path)
try:
    obj.link_to(_W())  # target: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__mkdir__mode_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__mkdir__mode_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__mkdir__mode_as_int_wrong"
# subject = "pathlib.Path.mkdir(mode: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.mkdir(mode: int); call it with the wrong type.

typeshed contract: mode is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pathlib import Path
obj = object.__new__(Path)
try:
    obj.mkdir("not_an_int")  # mode: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__move__target_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__move__target_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__move__target_as_StrPath_wrong"
# subject = "pathlib.Path.move(target: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.move(target: StrPath); call it with the wrong type.

typeshed contract: target is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import Path
obj = object.__new__(Path)
try:
    obj.move(_W())  # target: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__move__target_as__PathT_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__move__target_as__PathT_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__move__target_as__PathT_wrong"
# subject = "pathlib.Path.move(target: _PathT)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.move(target: _PathT); call it with the wrong type.

typeshed contract: target is _PathT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import Path
obj = object.__new__(Path)
try:
    obj.move(_W())  # target: _PathT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__move_into__target_dir_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__move_into__target_dir_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__move_into__target_dir_as_StrPath_wrong"
# subject = "pathlib.Path.move_into(target_dir: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.move_into(target_dir: StrPath); call it with the wrong type.

typeshed contract: target_dir is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import Path
obj = object.__new__(Path)
try:
    obj.move_into(_W())  # target_dir: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__move_into__target_dir_as__PathT_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__move_into__target_dir_as__PathT_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__move_into__target_dir_as__PathT_wrong"
# subject = "pathlib.Path.move_into(target_dir: _PathT)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.move_into(target_dir: _PathT); call it with the wrong type.

typeshed contract: target_dir is _PathT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import Path
obj = object.__new__(Path)
try:
    obj.move_into(_W())  # target_dir: _PathT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__open__mode_as_OpenBinaryModeReading_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__open__mode_as_OpenBinaryModeReading_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__open__mode_as_OpenBinaryModeReading_wrong"
# subject = "pathlib.Path.open(mode: OpenBinaryModeReading)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.open(mode: OpenBinaryModeReading); call it with the wrong type.

typeshed contract: mode is OpenBinaryModeReading. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import Path
obj = object.__new__(Path)
try:
    obj.open(_W())  # mode: OpenBinaryModeReading <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__open__mode_as_OpenBinaryModeUpdating_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__open__mode_as_OpenBinaryModeUpdating_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__open__mode_as_OpenBinaryModeUpdating_wrong"
# subject = "pathlib.Path.open(mode: OpenBinaryModeUpdating)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.open(mode: OpenBinaryModeUpdating); call it with the wrong type.

typeshed contract: mode is OpenBinaryModeUpdating. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import Path
obj = object.__new__(Path)
try:
    obj.open(_W())  # mode: OpenBinaryModeUpdating <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__open__mode_as_OpenBinaryModeWriting_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__open__mode_as_OpenBinaryModeWriting_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__open__mode_as_OpenBinaryModeWriting_wrong"
# subject = "pathlib.Path.open(mode: OpenBinaryModeWriting)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.open(mode: OpenBinaryModeWriting); call it with the wrong type.

typeshed contract: mode is OpenBinaryModeWriting. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import Path
obj = object.__new__(Path)
try:
    obj.open(_W())  # mode: OpenBinaryModeWriting <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__open__mode_as_OpenBinaryMode_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__open__mode_as_OpenBinaryMode_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__open__mode_as_OpenBinaryMode_wrong"
# subject = "pathlib.Path.open(mode: OpenBinaryMode)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.open(mode: OpenBinaryMode); call it with the wrong type.

typeshed contract: mode is OpenBinaryMode. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import Path
obj = object.__new__(Path)
try:
    obj.open(_W(), None)  # mode: OpenBinaryMode <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__open__mode_as_OpenTextMode_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__open__mode_as_OpenTextMode_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__open__mode_as_OpenTextMode_wrong"
# subject = "pathlib.Path.open(mode: OpenTextMode)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.open(mode: OpenTextMode); call it with the wrong type.

typeshed contract: mode is OpenTextMode. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import Path
obj = object.__new__(Path)
try:
    obj.open(_W())  # mode: OpenTextMode <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__open__mode_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__open__mode_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__open__mode_as_str_wrong"
# subject = "pathlib.Path.open(mode: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.open(mode: str); call it with the wrong type.

typeshed contract: mode is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pathlib import Path
obj = object.__new__(Path)
try:
    obj.open(12345)  # mode: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__read_text__encoding_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__read_text__encoding_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__read_text__encoding_as_typed_wrong"
# subject = "pathlib.Path.read_text(encoding: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.read_text(encoding: typed); call it with the wrong type.

typeshed contract: encoding is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import Path
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

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__rename__target_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__rename__target_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__rename__target_as_StrPath_wrong"
# subject = "pathlib.Path.rename(target: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.rename(target: StrPath); call it with the wrong type.

typeshed contract: target is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import Path
obj = object.__new__(Path)
try:
    obj.rename(_W())  # target: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__replace__target_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__replace__target_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__replace__target_as_StrPath_wrong"
# subject = "pathlib.Path.replace(target: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.replace(target: StrPath); call it with the wrong type.

typeshed contract: target is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import Path
obj = object.__new__(Path)
try:
    obj.replace(_W())  # target: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__resolve__strict_as_bool_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__resolve__strict_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__resolve__strict_as_bool_wrong"
# subject = "pathlib.Path.resolve(strict: bool)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.resolve(strict: bool); call it with the wrong type.

typeshed contract: strict is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pathlib import Path
obj = object.__new__(Path)
try:
    obj.resolve("not_a_bool")  # strict: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__rglob__pattern_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__rglob__pattern_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__rglob__pattern_as_str_wrong"
# subject = "pathlib.Path.rglob(pattern: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.rglob(pattern: str); call it with the wrong type.

typeshed contract: pattern is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pathlib import Path
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

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__samefile__other_path_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__samefile__other_path_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__samefile__other_path_as_StrPath_wrong"
# subject = "pathlib.Path.samefile(other_path: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.samefile(other_path: StrPath); call it with the wrong type.

typeshed contract: other_path is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import Path
obj = object.__new__(Path)
try:
    obj.samefile(_W())  # other_path: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__symlink_to__target_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__symlink_to__target_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__symlink_to__target_as_StrOrBytesPath_wrong"
# subject = "pathlib.Path.symlink_to(target: StrOrBytesPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.symlink_to(target: StrOrBytesPath); call it with the wrong type.

typeshed contract: target is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import Path
obj = object.__new__(Path)
try:
    obj.symlink_to(_W())  # target: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__touch__mode_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__touch__mode_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__touch__mode_as_int_wrong"
# subject = "pathlib.Path.touch(mode: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.touch(mode: int); call it with the wrong type.

typeshed contract: mode is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pathlib import Path
obj = object.__new__(Path)
try:
    obj.touch("not_an_int")  # mode: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__unlink__missing_ok_as_bool_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__unlink__missing_ok_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__unlink__missing_ok_as_bool_wrong"
# subject = "pathlib.Path.unlink(missing_ok: bool)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.unlink(missing_ok: bool); call it with the wrong type.

typeshed contract: missing_ok is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pathlib import Path
obj = object.__new__(Path)
try:
    obj.unlink("not_a_bool")  # missing_ok: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__walk__top_down_as_bool_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__walk__top_down_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__walk__top_down_as_bool_wrong"
# subject = "pathlib.Path.walk(top_down: bool)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.walk(top_down: bool); call it with the wrong type.

typeshed contract: top_down is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pathlib import Path
obj = object.__new__(Path)
try:
    obj.walk("not_a_bool")  # top_down: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__write_bytes__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__write_bytes__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__write_bytes__data_as_ReadableBuffer_wrong"
# subject = "pathlib.Path.write_bytes(data: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.write_bytes(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import Path
obj = object.__new__(Path)
try:
    obj.write_bytes(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/Path__write_text__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_Path__write_text__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__write_text__data_as_str_wrong"
# subject = "pathlib.Path.write_text(data: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.write_text(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pathlib import Path
obj = object.__new__(Path)
try:
    obj.write_text(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/PurePath____ge____other_as_PurePath_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_PurePath____ge____other_as_PurePath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "PurePath____ge____other_as_PurePath_wrong"
# subject = "pathlib.PurePath.__ge__(other: PurePath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.PurePath.__ge__(other: PurePath); call it with the wrong type.

typeshed contract: other is PurePath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import PurePath
obj = object.__new__(PurePath)
try:
    obj.__ge__(_W())  # other: PurePath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/PurePath____gt____other_as_PurePath_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_PurePath____gt____other_as_PurePath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "PurePath____gt____other_as_PurePath_wrong"
# subject = "pathlib.PurePath.__gt__(other: PurePath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.PurePath.__gt__(other: PurePath); call it with the wrong type.

typeshed contract: other is PurePath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import PurePath
obj = object.__new__(PurePath)
try:
    obj.__gt__(_W())  # other: PurePath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/PurePath____le____other_as_PurePath_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_PurePath____le____other_as_PurePath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "PurePath____le____other_as_PurePath_wrong"
# subject = "pathlib.PurePath.__le__(other: PurePath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.PurePath.__le__(other: PurePath); call it with the wrong type.

typeshed contract: other is PurePath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import PurePath
obj = object.__new__(PurePath)
try:
    obj.__le__(_W())  # other: PurePath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/PurePath____lt____other_as_PurePath_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_PurePath____lt____other_as_PurePath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "PurePath____lt____other_as_PurePath_wrong"
# subject = "pathlib.PurePath.__lt__(other: PurePath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.PurePath.__lt__(other: PurePath); call it with the wrong type.

typeshed contract: other is PurePath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import PurePath
obj = object.__new__(PurePath)
try:
    obj.__lt__(_W())  # other: PurePath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/PurePath____rtruediv____key_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_PurePath____rtruediv____key_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "PurePath____rtruediv____key_as_StrPath_wrong"
# subject = "pathlib.PurePath.__rtruediv__(key: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.PurePath.__rtruediv__(key: StrPath); call it with the wrong type.

typeshed contract: key is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import PurePath
obj = object.__new__(PurePath)
try:
    obj.__rtruediv__(_W())  # key: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/PurePath____truediv____key_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_PurePath____truediv____key_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "PurePath____truediv____key_as_StrPath_wrong"
# subject = "pathlib.PurePath.__truediv__(key: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.PurePath.__truediv__(key: StrPath); call it with the wrong type.

typeshed contract: key is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import PurePath
obj = object.__new__(PurePath)
try:
    obj.__truediv__(_W())  # key: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/PurePath__full_match__pattern_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_PurePath__full_match__pattern_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "PurePath__full_match__pattern_as_StrPath_wrong"
# subject = "pathlib.PurePath.full_match(pattern: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.PurePath.full_match(pattern: StrPath); call it with the wrong type.

typeshed contract: pattern is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import PurePath
obj = object.__new__(PurePath)
try:
    obj.full_match(_W())  # pattern: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/PurePath__is_relative_to__other_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_PurePath__is_relative_to__other_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "PurePath__is_relative_to__other_as_StrPath_wrong"
# subject = "pathlib.PurePath.is_relative_to(other: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.PurePath.is_relative_to(other: StrPath); call it with the wrong type.

typeshed contract: other is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import PurePath
obj = object.__new__(PurePath)
try:
    obj.is_relative_to(_W())  # other: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/PurePath__match__path_pattern_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_PurePath__match__path_pattern_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "PurePath__match__path_pattern_as_str_wrong"
# subject = "pathlib.PurePath.match(path_pattern: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.PurePath.match(path_pattern: str); call it with the wrong type.

typeshed contract: path_pattern is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pathlib import PurePath
obj = object.__new__(PurePath)
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

/// Ported from `tests/cpython/type/std-libs/pathlib/PurePath__relative_to__other_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_PurePath__relative_to__other_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "PurePath__relative_to__other_as_StrPath_wrong"
# subject = "pathlib.PurePath.relative_to(other: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.PurePath.relative_to(other: StrPath); call it with the wrong type.

typeshed contract: other is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import PurePath
obj = object.__new__(PurePath)
try:
    obj.relative_to(_W())  # other: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/PurePath__with_name__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_PurePath__with_name__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "PurePath__with_name__name_as_str_wrong"
# subject = "pathlib.PurePath.with_name(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.PurePath.with_name(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pathlib import PurePath
obj = object.__new__(PurePath)
try:
    obj.with_name(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/PurePath__with_stem__stem_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_PurePath__with_stem__stem_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "PurePath__with_stem__stem_as_str_wrong"
# subject = "pathlib.PurePath.with_stem(stem: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.PurePath.with_stem(stem: str); call it with the wrong type.

typeshed contract: stem is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pathlib import PurePath
obj = object.__new__(PurePath)
try:
    obj.with_stem(12345)  # stem: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pathlib/PurePath__with_suffix__suffix_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pathlib_PurePath__with_suffix__suffix_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "PurePath__with_suffix__suffix_as_str_wrong"
# subject = "pathlib.PurePath.with_suffix(suffix: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.PurePath.with_suffix(suffix: str); call it with the wrong type.

typeshed contract: suffix is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pathlib import PurePath
obj = object.__new__(PurePath)
try:
    obj.with_suffix(12345)  # suffix: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
