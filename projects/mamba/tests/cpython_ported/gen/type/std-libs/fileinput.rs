use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/fileinput/FileInput____exit____type_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_fileinput_FileInput____exit____type_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileinput"
# dimension = "type"
# case = "FileInput____exit____type_as_typed_wrong"
# subject = "fileinput.FileInput.__exit__(type: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/fileinput.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: fileinput.FileInput.__exit__(type: typed); call it with the wrong type.

typeshed contract: type is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from fileinput import FileInput
obj = object.__new__(FileInput)
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

/// Ported from `tests/cpython/type/std-libs/fileinput/FileInput____getitem____i_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_fileinput_FileInput____getitem____i_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileinput"
# dimension = "type"
# case = "FileInput____getitem____i_as_int_wrong"
# subject = "fileinput.FileInput.__getitem__(i: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/fileinput.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: fileinput.FileInput.__getitem__(i: int); call it with the wrong type.

typeshed contract: i is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from fileinput import FileInput
obj = object.__new__(FileInput)
try:
    obj.__getitem__("not_an_int")  # i: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/fileinput/FileInput__init__files_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_fileinput_FileInput__init__files_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileinput"
# dimension = "type"
# case = "FileInput__init__files_as_typed_wrong"
# subject = "fileinput.FileInput.__init__(files: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/fileinput.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: fileinput.FileInput.__init__(files: typed); call it with the wrong type.

typeshed contract: files is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from fileinput import FileInput
try:
    FileInput(_W())  # files: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/fileinput/hook_compressed__filename_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_fileinput_hook_compressed__filename_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileinput"
# dimension = "type"
# case = "hook_compressed__filename_as_StrOrBytesPath_wrong"
# subject = "fileinput.hook_compressed(filename: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/fileinput.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: fileinput.hook_compressed(filename: StrOrBytesPath); call it with the wrong type.

typeshed contract: filename is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from fileinput import hook_compressed
try:
    hook_compressed(_W(), "")  # filename: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/fileinput/hook_encoded__encoding_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_fileinput_hook_encoded__encoding_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileinput"
# dimension = "type"
# case = "hook_encoded__encoding_as_str_wrong"
# subject = "fileinput.hook_encoded(encoding: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/fileinput.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: fileinput.hook_encoded(encoding: str); call it with the wrong type.

typeshed contract: encoding is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from fileinput import hook_encoded
try:
    hook_encoded(12345)  # encoding: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/fileinput/input__files_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_fileinput_input__files_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileinput"
# dimension = "type"
# case = "input__files_as_typed_wrong"
# subject = "fileinput.input(files: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/fileinput.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: fileinput.input(files: typed); call it with the wrong type.

typeshed contract: files is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from fileinput import input
try:
    input(_W())  # files: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
