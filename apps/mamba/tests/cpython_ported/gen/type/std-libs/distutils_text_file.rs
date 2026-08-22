use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/distutils_text_file/TextFile__init__filename_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_text_file_TextFile__init__filename_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_text_file"
# dimension = "type"
# case = "TextFile__init__filename_as_typed_wrong"
# subject = "distutils.text_file.TextFile.__init__(filename: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/text_file.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.text_file.TextFile.__init__(filename: typed); call it with the wrong type.

typeshed contract: filename is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.text_file import TextFile
try:
    TextFile(_W())  # filename: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_text_file/TextFile__open__filename_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_text_file_TextFile__open__filename_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_text_file"
# dimension = "type"
# case = "TextFile__open__filename_as_str_wrong"
# subject = "distutils.text_file.TextFile.open(filename: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/text_file.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.text_file.TextFile.open(filename: str); call it with the wrong type.

typeshed contract: filename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.text_file import TextFile
obj = object.__new__(TextFile)
try:
    obj.open(12345)  # filename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_text_file/TextFile__unreadline__line_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_text_file_TextFile__unreadline__line_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_text_file"
# dimension = "type"
# case = "TextFile__unreadline__line_as_str_wrong"
# subject = "distutils.text_file.TextFile.unreadline(line: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/text_file.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.text_file.TextFile.unreadline(line: str); call it with the wrong type.

typeshed contract: line is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.text_file import TextFile
obj = object.__new__(TextFile)
try:
    obj.unreadline(12345)  # line: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_text_file/TextFile__warn__msg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_text_file_TextFile__warn__msg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_text_file"
# dimension = "type"
# case = "TextFile__warn__msg_as_str_wrong"
# subject = "distutils.text_file.TextFile.warn(msg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/text_file.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.text_file.TextFile.warn(msg: str); call it with the wrong type.

typeshed contract: msg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.text_file import TextFile
obj = object.__new__(TextFile)
try:
    obj.warn(12345)  # msg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
