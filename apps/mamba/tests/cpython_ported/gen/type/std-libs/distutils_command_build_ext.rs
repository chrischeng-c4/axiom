use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/distutils_command_build_ext/build_ext__get_ext_filename__ext_name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_command_build_ext_build_ext__get_ext_filename__ext_name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_command_build_ext"
# dimension = "type"
# case = "build_ext__get_ext_filename__ext_name_as_str_wrong"
# subject = "distutils.command.build_ext.build_ext.get_ext_filename(ext_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/command/build_ext.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.command.build_ext.build_ext.get_ext_filename(ext_name: str); call it with the wrong type.

typeshed contract: ext_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.command.build_ext import build_ext
obj = object.__new__(build_ext)
try:
    obj.get_ext_filename(12345)  # ext_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_command_build_ext/build_ext__get_ext_fullname__ext_name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_command_build_ext_build_ext__get_ext_fullname__ext_name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_command_build_ext"
# dimension = "type"
# case = "build_ext__get_ext_fullname__ext_name_as_str_wrong"
# subject = "distutils.command.build_ext.build_ext.get_ext_fullname(ext_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/command/build_ext.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.command.build_ext.build_ext.get_ext_fullname(ext_name: str); call it with the wrong type.

typeshed contract: ext_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.command.build_ext import build_ext
obj = object.__new__(build_ext)
try:
    obj.get_ext_fullname(12345)  # ext_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_command_build_ext/build_ext__get_ext_fullpath__ext_name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_command_build_ext_build_ext__get_ext_fullpath__ext_name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_command_build_ext"
# dimension = "type"
# case = "build_ext__get_ext_fullpath__ext_name_as_str_wrong"
# subject = "distutils.command.build_ext.build_ext.get_ext_fullpath(ext_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/command/build_ext.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.command.build_ext.build_ext.get_ext_fullpath(ext_name: str); call it with the wrong type.

typeshed contract: ext_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.command.build_ext import build_ext
obj = object.__new__(build_ext)
try:
    obj.get_ext_fullpath(12345)  # ext_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
