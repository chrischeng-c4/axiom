use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_osx_support/compiler_fixup__compiler_so_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__osx_support_compiler_fixup__compiler_so_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_osx_support"
# dimension = "type"
# case = "compiler_fixup__compiler_so_as_Iterable_wrong"
# subject = "_osx_support.compiler_fixup(compiler_so: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_osx_support.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _osx_support.compiler_fixup(compiler_so: Iterable); call it with the wrong type.

typeshed contract: compiler_so is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _osx_support import compiler_fixup
try:
    compiler_fixup(_W(), None)  # compiler_so: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_osx_support/customize_compiler___config_vars_as_dict_wrong.py`.
#[test]
fn test_gen_type_std_libs__osx_support_customize_compiler___config_vars_as_dict_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_osx_support"
# dimension = "type"
# case = "customize_compiler___config_vars_as_dict_wrong"
# subject = "_osx_support.customize_compiler(_config_vars: dict)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_osx_support.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _osx_support.customize_compiler(_config_vars: dict); call it with the wrong type.

typeshed contract: _config_vars is dict. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _osx_support import customize_compiler
try:
    customize_compiler(12345)  # _config_vars: dict <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_osx_support/customize_config_vars___config_vars_as_dict_wrong.py`.
#[test]
fn test_gen_type_std_libs__osx_support_customize_config_vars___config_vars_as_dict_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_osx_support"
# dimension = "type"
# case = "customize_config_vars___config_vars_as_dict_wrong"
# subject = "_osx_support.customize_config_vars(_config_vars: dict)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_osx_support.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _osx_support.customize_config_vars(_config_vars: dict); call it with the wrong type.

typeshed contract: _config_vars is dict. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _osx_support import customize_config_vars
try:
    customize_config_vars(12345)  # _config_vars: dict <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_osx_support/get_platform_osx___config_vars_as_dict_wrong.py`.
#[test]
fn test_gen_type_std_libs__osx_support_get_platform_osx___config_vars_as_dict_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_osx_support"
# dimension = "type"
# case = "get_platform_osx___config_vars_as_dict_wrong"
# subject = "_osx_support.get_platform_osx(_config_vars: dict)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_osx_support.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _osx_support.get_platform_osx(_config_vars: dict); call it with the wrong type.

typeshed contract: _config_vars is dict. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _osx_support import get_platform_osx
try:
    get_platform_osx(12345, None, None, None)  # _config_vars: dict <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
