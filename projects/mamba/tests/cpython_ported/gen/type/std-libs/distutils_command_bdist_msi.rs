use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/distutils_command_bdist_msi/PyDialog__back__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_command_bdist_msi_PyDialog__back__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_command_bdist_msi"
# dimension = "type"
# case = "PyDialog__back__name_as_str_wrong"
# subject = "distutils.command.bdist_msi.PyDialog.back(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/command/bdist_msi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.command.bdist_msi.PyDialog.back(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.command.bdist_msi import PyDialog
obj = object.__new__(PyDialog)
try:
    obj.back(None, None, 12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_command_bdist_msi/PyDialog__cancel__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_command_bdist_msi_PyDialog__cancel__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_command_bdist_msi"
# dimension = "type"
# case = "PyDialog__cancel__name_as_str_wrong"
# subject = "distutils.command.bdist_msi.PyDialog.cancel(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/command/bdist_msi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.command.bdist_msi.PyDialog.cancel(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.command.bdist_msi import PyDialog
obj = object.__new__(PyDialog)
try:
    obj.cancel(None, None, 12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_command_bdist_msi/PyDialog__next__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_command_bdist_msi_PyDialog__next__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_command_bdist_msi"
# dimension = "type"
# case = "PyDialog__next__name_as_str_wrong"
# subject = "distutils.command.bdist_msi.PyDialog.next(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/command/bdist_msi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.command.bdist_msi.PyDialog.next(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.command.bdist_msi import PyDialog
obj = object.__new__(PyDialog)
try:
    obj.next(None, None, 12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
