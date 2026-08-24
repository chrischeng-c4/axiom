use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/mimetypes/MimeTypes__add_type__type_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mimetypes_MimeTypes__add_type__type_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "type"
# case = "MimeTypes__add_type__type_as_str_wrong"
# subject = "mimetypes.MimeTypes.add_type(type: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mimetypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mimetypes.MimeTypes.add_type(type: str); call it with the wrong type.

typeshed contract: type is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mimetypes import MimeTypes
obj = object.__new__(MimeTypes)
try:
    obj.add_type(12345, "")  # type: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mimetypes/MimeTypes__guess_all_extensions__type_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mimetypes_MimeTypes__guess_all_extensions__type_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "type"
# case = "MimeTypes__guess_all_extensions__type_as_str_wrong"
# subject = "mimetypes.MimeTypes.guess_all_extensions(type: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mimetypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mimetypes.MimeTypes.guess_all_extensions(type: str); call it with the wrong type.

typeshed contract: type is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mimetypes import MimeTypes
obj = object.__new__(MimeTypes)
try:
    obj.guess_all_extensions(12345)  # type: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mimetypes/MimeTypes__guess_extension__type_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mimetypes_MimeTypes__guess_extension__type_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "type"
# case = "MimeTypes__guess_extension__type_as_str_wrong"
# subject = "mimetypes.MimeTypes.guess_extension(type: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mimetypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mimetypes.MimeTypes.guess_extension(type: str); call it with the wrong type.

typeshed contract: type is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mimetypes import MimeTypes
obj = object.__new__(MimeTypes)
try:
    obj.guess_extension(12345)  # type: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mimetypes/MimeTypes__guess_file_type__path_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_mimetypes_MimeTypes__guess_file_type__path_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "type"
# case = "MimeTypes__guess_file_type__path_as_StrPath_wrong"
# subject = "mimetypes.MimeTypes.guess_file_type(path: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mimetypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mimetypes.MimeTypes.guess_file_type(path: StrPath); call it with the wrong type.

typeshed contract: path is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mimetypes import MimeTypes
obj = object.__new__(MimeTypes)
try:
    obj.guess_file_type(_W())  # path: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mimetypes/MimeTypes__guess_type__url_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_mimetypes_MimeTypes__guess_type__url_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "type"
# case = "MimeTypes__guess_type__url_as_StrPath_wrong"
# subject = "mimetypes.MimeTypes.guess_type(url: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mimetypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mimetypes.MimeTypes.guess_type(url: StrPath); call it with the wrong type.

typeshed contract: url is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mimetypes import MimeTypes
obj = object.__new__(MimeTypes)
try:
    obj.guess_type(_W())  # url: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mimetypes/MimeTypes__init__filenames_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_mimetypes_MimeTypes__init__filenames_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "type"
# case = "MimeTypes__init__filenames_as_Iterable_wrong"
# subject = "mimetypes.MimeTypes.__init__(filenames: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mimetypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mimetypes.MimeTypes.__init__(filenames: Iterable); call it with the wrong type.

typeshed contract: filenames is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mimetypes import MimeTypes
try:
    MimeTypes(_W())  # filenames: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mimetypes/MimeTypes__read__filename_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_mimetypes_MimeTypes__read__filename_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "type"
# case = "MimeTypes__read__filename_as_StrPath_wrong"
# subject = "mimetypes.MimeTypes.read(filename: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mimetypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mimetypes.MimeTypes.read(filename: StrPath); call it with the wrong type.

typeshed contract: filename is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mimetypes import MimeTypes
obj = object.__new__(MimeTypes)
try:
    obj.read(_W())  # filename: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mimetypes/MimeTypes__read_windows_registry__strict_as_bool_wrong.py`.
#[test]
fn test_gen_type_std_libs_mimetypes_MimeTypes__read_windows_registry__strict_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "type"
# case = "MimeTypes__read_windows_registry__strict_as_bool_wrong"
# subject = "mimetypes.MimeTypes.read_windows_registry(strict: bool)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mimetypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mimetypes.MimeTypes.read_windows_registry(strict: bool); call it with the wrong type.

typeshed contract: strict is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mimetypes import MimeTypes
obj = object.__new__(MimeTypes)
try:
    obj.read_windows_registry("not_a_bool")  # strict: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mimetypes/MimeTypes__readfp__fp_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_mimetypes_MimeTypes__readfp__fp_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "type"
# case = "MimeTypes__readfp__fp_as_IO_wrong"
# subject = "mimetypes.MimeTypes.readfp(fp: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mimetypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mimetypes.MimeTypes.readfp(fp: IO); call it with the wrong type.

typeshed contract: fp is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mimetypes import MimeTypes
obj = object.__new__(MimeTypes)
try:
    obj.readfp(_W())  # fp: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mimetypes/add_type__type_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mimetypes_add_type__type_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "type"
# case = "add_type__type_as_str_wrong"
# subject = "mimetypes.add_type(type: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mimetypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mimetypes.add_type(type: str); call it with the wrong type.

typeshed contract: type is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mimetypes import add_type
try:
    add_type(12345, "")  # type: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mimetypes/guess_all_extensions__type_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mimetypes_guess_all_extensions__type_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "type"
# case = "guess_all_extensions__type_as_str_wrong"
# subject = "mimetypes.guess_all_extensions(type: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mimetypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mimetypes.guess_all_extensions(type: str); call it with the wrong type.

typeshed contract: type is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mimetypes import guess_all_extensions
try:
    guess_all_extensions(12345)  # type: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mimetypes/guess_extension__type_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mimetypes_guess_extension__type_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "type"
# case = "guess_extension__type_as_str_wrong"
# subject = "mimetypes.guess_extension(type: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mimetypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mimetypes.guess_extension(type: str); call it with the wrong type.

typeshed contract: type is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mimetypes import guess_extension
try:
    guess_extension(12345)  # type: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mimetypes/guess_file_type__path_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_mimetypes_guess_file_type__path_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "type"
# case = "guess_file_type__path_as_StrPath_wrong"
# subject = "mimetypes.guess_file_type(path: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mimetypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mimetypes.guess_file_type(path: StrPath); call it with the wrong type.

typeshed contract: path is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mimetypes import guess_file_type
try:
    guess_file_type(_W())  # path: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mimetypes/guess_type__url_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_mimetypes_guess_type__url_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "type"
# case = "guess_type__url_as_StrPath_wrong"
# subject = "mimetypes.guess_type(url: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mimetypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mimetypes.guess_type(url: StrPath); call it with the wrong type.

typeshed contract: url is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mimetypes import guess_type
try:
    guess_type(_W())  # url: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mimetypes/init__files_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_mimetypes_init__files_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "type"
# case = "init__files_as_typed_wrong"
# subject = "mimetypes.init(files: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mimetypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mimetypes.init(files: typed); call it with the wrong type.

typeshed contract: files is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mimetypes import init
try:
    init(_W())  # files: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mimetypes/read_mime_types__file_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_mimetypes_read_mime_types__file_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "type"
# case = "read_mime_types__file_as_StrPath_wrong"
# subject = "mimetypes.read_mime_types(file: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mimetypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mimetypes.read_mime_types(file: StrPath); call it with the wrong type.

typeshed contract: file is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mimetypes import read_mime_types
try:
    read_mime_types(_W())  # file: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
