use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/zipfile/CompleteDirs__make__source_as_ZipFile_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_CompleteDirs__make__source_as_ZipFile_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "CompleteDirs__make__source_as_ZipFile_wrong"
# subject = "zipfile.CompleteDirs.make(source: ZipFile)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.CompleteDirs.make(source: ZipFile); call it with the wrong type.

typeshed contract: source is ZipFile. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import CompleteDirs
try:
    CompleteDirs.make(_W())  # source: ZipFile <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/CompleteDirs__make__source_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_CompleteDirs__make__source_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "CompleteDirs__make__source_as_typed_wrong"
# subject = "zipfile.CompleteDirs.make(source: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.CompleteDirs.make(source: typed); call it with the wrong type.

typeshed contract: source is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import CompleteDirs
try:
    CompleteDirs.make(_W())  # source: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/CompleteDirs__resolve_dir__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_CompleteDirs__resolve_dir__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "CompleteDirs__resolve_dir__name_as_str_wrong"
# subject = "zipfile.CompleteDirs.resolve_dir(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.CompleteDirs.resolve_dir(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipfile import CompleteDirs
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

/// Ported from `tests/cpython/type/std-libs/zipfile/Path____truediv____add_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_Path____truediv____add_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "Path____truediv____add_as_StrPath_wrong"
# subject = "zipfile.Path.__truediv__(add: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.Path.__truediv__(add: StrPath); call it with the wrong type.

typeshed contract: add is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import Path
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

/// Ported from `tests/cpython/type/std-libs/zipfile/Path__init__root_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_Path__init__root_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "Path__init__root_as_typed_wrong"
# subject = "zipfile.Path.__init__(root: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.Path.__init__(root: typed); call it with the wrong type.

typeshed contract: root is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import Path
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

/// Ported from `tests/cpython/type/std-libs/zipfile/Path__open__mode_as_Literal_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_Path__open__mode_as_Literal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "Path__open__mode_as_Literal_wrong"
# subject = "zipfile.Path.open(mode: Literal)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.Path.open(mode: Literal); call it with the wrong type.

typeshed contract: mode is Literal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import Path
obj = object.__new__(Path)
try:
    obj.open(_W())  # mode: Literal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/Path__read_text__encoding_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_Path__read_text__encoding_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "Path__read_text__encoding_as_typed_wrong"
# subject = "zipfile.Path.read_text(encoding: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.Path.read_text(encoding: typed); call it with the wrong type.

typeshed contract: encoding is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import Path
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

/// Ported from `tests/cpython/type/std-libs/zipfile/PyZipFile__init__file_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_PyZipFile__init__file_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "PyZipFile__init__file_as_typed_wrong"
# subject = "zipfile.PyZipFile.__init__(file: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.PyZipFile.__init__(file: typed); call it with the wrong type.

typeshed contract: file is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import PyZipFile
try:
    PyZipFile(_W())  # file: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/PyZipFile__writepy__pathname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_PyZipFile__writepy__pathname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "PyZipFile__writepy__pathname_as_str_wrong"
# subject = "zipfile.PyZipFile.writepy(pathname: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.PyZipFile.writepy(pathname: str); call it with the wrong type.

typeshed contract: pathname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipfile import PyZipFile
obj = object.__new__(PyZipFile)
try:
    obj.writepy(12345)  # pathname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/ZipExtFile__init__fileobj_as__ClosableZipStream_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_ZipExtFile__init__fileobj_as__ClosableZipStream_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipExtFile__init__fileobj_as__ClosableZipStream_wrong"
# subject = "zipfile.ZipExtFile.__init__(fileobj: _ClosableZipStream)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipExtFile.__init__(fileobj: _ClosableZipStream); call it with the wrong type.

typeshed contract: fileobj is _ClosableZipStream. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import ZipExtFile
try:
    ZipExtFile(_W(), None, None, None, None)  # fileobj: _ClosableZipStream <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/ZipExtFile__init__fileobj_as__ZipStream_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_ZipExtFile__init__fileobj_as__ZipStream_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipExtFile__init__fileobj_as__ZipStream_wrong"
# subject = "zipfile.ZipExtFile.__init__(fileobj: _ZipStream)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipExtFile.__init__(fileobj: _ZipStream); call it with the wrong type.

typeshed contract: fileobj is _ZipStream. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import ZipExtFile
try:
    ZipExtFile(_W(), None, None)  # fileobj: _ZipStream <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/ZipExtFile__peek__n_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_ZipExtFile__peek__n_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipExtFile__peek__n_as_int_wrong"
# subject = "zipfile.ZipExtFile.peek(n: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipExtFile.peek(n: int); call it with the wrong type.

typeshed contract: n is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipfile import ZipExtFile
obj = object.__new__(ZipExtFile)
try:
    obj.peek("not_an_int")  # n: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/ZipExtFile__read1__n_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_ZipExtFile__read1__n_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipExtFile__read1__n_as_typed_wrong"
# subject = "zipfile.ZipExtFile.read1(n: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipExtFile.read1(n: typed); call it with the wrong type.

typeshed contract: n is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import ZipExtFile
obj = object.__new__(ZipExtFile)
try:
    obj.read1(_W())  # n: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/ZipExtFile__read__n_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_ZipExtFile__read__n_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipExtFile__read__n_as_typed_wrong"
# subject = "zipfile.ZipExtFile.read(n: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipExtFile.read(n: typed); call it with the wrong type.

typeshed contract: n is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import ZipExtFile
obj = object.__new__(ZipExtFile)
try:
    obj.read(_W())  # n: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/ZipExtFile__readline__limit_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_ZipExtFile__readline__limit_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipExtFile__readline__limit_as_int_wrong"
# subject = "zipfile.ZipExtFile.readline(limit: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipExtFile.readline(limit: int); call it with the wrong type.

typeshed contract: limit is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipfile import ZipExtFile
obj = object.__new__(ZipExtFile)
try:
    obj.readline("not_an_int")  # limit: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/ZipExtFile__seek__offset_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_ZipExtFile__seek__offset_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipExtFile__seek__offset_as_int_wrong"
# subject = "zipfile.ZipExtFile.seek(offset: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipExtFile.seek(offset: int); call it with the wrong type.

typeshed contract: offset is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipfile import ZipExtFile
obj = object.__new__(ZipExtFile)
try:
    obj.seek("not_an_int")  # offset: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/ZipFile____exit____type_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_ZipFile____exit____type_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipFile____exit____type_as_typed_wrong"
# subject = "zipfile.ZipFile.__exit__(type: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipFile.__exit__(type: typed); call it with the wrong type.

typeshed contract: type is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import ZipFile
obj = object.__new__(ZipFile)
try:
    obj.__exit__(_W(), None, None)  # type: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/ZipFile__extract__member_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_ZipFile__extract__member_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipFile__extract__member_as_typed_wrong"
# subject = "zipfile.ZipFile.extract(member: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipFile.extract(member: typed); call it with the wrong type.

typeshed contract: member is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import ZipFile
obj = object.__new__(ZipFile)
try:
    obj.extract(_W())  # member: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/ZipFile__extractall__path_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_ZipFile__extractall__path_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipFile__extractall__path_as_typed_wrong"
# subject = "zipfile.ZipFile.extractall(path: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipFile.extractall(path: typed); call it with the wrong type.

typeshed contract: path is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import ZipFile
obj = object.__new__(ZipFile)
try:
    obj.extractall(_W())  # path: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/ZipFile__getinfo__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_ZipFile__getinfo__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipFile__getinfo__name_as_str_wrong"
# subject = "zipfile.ZipFile.getinfo(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipFile.getinfo(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipfile import ZipFile
obj = object.__new__(ZipFile)
try:
    obj.getinfo(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/ZipFile__init__file_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_ZipFile__init__file_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipFile__init__file_as_typed_wrong"
# subject = "zipfile.ZipFile.__init__(file: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipFile.__init__(file: typed); call it with the wrong type.

typeshed contract: file is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import ZipFile
try:
    ZipFile(_W())  # file: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/ZipFile__mkdir__zinfo_or_directory_name_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_ZipFile__mkdir__zinfo_or_directory_name_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipFile__mkdir__zinfo_or_directory_name_as_typed_wrong"
# subject = "zipfile.ZipFile.mkdir(zinfo_or_directory_name: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipFile.mkdir(zinfo_or_directory_name: typed); call it with the wrong type.

typeshed contract: zinfo_or_directory_name is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import ZipFile
obj = object.__new__(ZipFile)
try:
    obj.mkdir(_W())  # zinfo_or_directory_name: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/ZipFile__open__name_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_ZipFile__open__name_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipFile__open__name_as_typed_wrong"
# subject = "zipfile.ZipFile.open(name: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipFile.open(name: typed); call it with the wrong type.

typeshed contract: name is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import ZipFile
obj = object.__new__(ZipFile)
try:
    obj.open(_W())  # name: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/ZipFile__printdir__file_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_ZipFile__printdir__file_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipFile__printdir__file_as_typed_wrong"
# subject = "zipfile.ZipFile.printdir(file: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipFile.printdir(file: typed); call it with the wrong type.

typeshed contract: file is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import ZipFile
obj = object.__new__(ZipFile)
try:
    obj.printdir(_W())  # file: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/ZipFile__read__name_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_ZipFile__read__name_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipFile__read__name_as_typed_wrong"
# subject = "zipfile.ZipFile.read(name: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipFile.read(name: typed); call it with the wrong type.

typeshed contract: name is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import ZipFile
obj = object.__new__(ZipFile)
try:
    obj.read(_W())  # name: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/ZipFile__setpassword__pwd_as_bytes_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_ZipFile__setpassword__pwd_as_bytes_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipFile__setpassword__pwd_as_bytes_wrong"
# subject = "zipfile.ZipFile.setpassword(pwd: bytes)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipFile.setpassword(pwd: bytes); call it with the wrong type.

typeshed contract: pwd is bytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipfile import ZipFile
obj = object.__new__(ZipFile)
try:
    obj.setpassword(12345)  # pwd: bytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/ZipFile__write__filename_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_ZipFile__write__filename_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipFile__write__filename_as_StrPath_wrong"
# subject = "zipfile.ZipFile.write(filename: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipFile.write(filename: StrPath); call it with the wrong type.

typeshed contract: filename is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import ZipFile
obj = object.__new__(ZipFile)
try:
    obj.write(_W())  # filename: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/ZipFile__writestr__zinfo_or_arcname_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_ZipFile__writestr__zinfo_or_arcname_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipFile__writestr__zinfo_or_arcname_as_typed_wrong"
# subject = "zipfile.ZipFile.writestr(zinfo_or_arcname: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipFile.writestr(zinfo_or_arcname: typed); call it with the wrong type.

typeshed contract: zinfo_or_arcname is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import ZipFile
obj = object.__new__(ZipFile)
try:
    obj.writestr(_W(), None)  # zinfo_or_arcname: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/ZipInfo__FileHeader__zip64_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_ZipInfo__FileHeader__zip64_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipInfo__FileHeader__zip64_as_typed_wrong"
# subject = "zipfile.ZipInfo.FileHeader(zip64: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipInfo.FileHeader(zip64: typed); call it with the wrong type.

typeshed contract: zip64 is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import ZipInfo
obj = object.__new__(ZipInfo)
try:
    obj.FileHeader(_W())  # zip64: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/ZipInfo__from_file__filename_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_ZipInfo__from_file__filename_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipInfo__from_file__filename_as_StrPath_wrong"
# subject = "zipfile.ZipInfo.from_file(filename: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipInfo.from_file(filename: StrPath); call it with the wrong type.

typeshed contract: filename is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import ZipInfo
try:
    ZipInfo.from_file(_W())  # filename: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/ZipInfo__init__filename_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_ZipInfo__init__filename_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipInfo__init__filename_as_str_wrong"
# subject = "zipfile.ZipInfo.__init__(filename: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipInfo.__init__(filename: str); call it with the wrong type.

typeshed contract: filename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipfile import ZipInfo
try:
    ZipInfo(12345)  # filename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipfile/is_zipfile__filename_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipfile_is_zipfile__filename_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "is_zipfile__filename_as_typed_wrong"
# subject = "zipfile.is_zipfile(filename: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.is_zipfile(filename: typed); call it with the wrong type.

typeshed contract: filename is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import is_zipfile
try:
    is_zipfile(_W())  # filename: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
