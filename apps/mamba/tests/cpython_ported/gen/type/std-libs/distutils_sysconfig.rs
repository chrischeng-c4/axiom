use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/distutils_sysconfig/customize_compiler__compiler_as_CCompiler_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_sysconfig_customize_compiler__compiler_as_CCompiler_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_sysconfig"
# dimension = "type"
# case = "customize_compiler__compiler_as_CCompiler_wrong"
# subject = "distutils.sysconfig.customize_compiler(compiler: CCompiler)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/sysconfig.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.sysconfig.customize_compiler(compiler: CCompiler); call it with the wrong type.

typeshed contract: compiler is CCompiler. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.sysconfig import customize_compiler
try:
    customize_compiler(_W())  # compiler: CCompiler <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_sysconfig/expand_makefile_vars__s_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_sysconfig_expand_makefile_vars__s_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_sysconfig"
# dimension = "type"
# case = "expand_makefile_vars__s_as_str_wrong"
# subject = "distutils.sysconfig.expand_makefile_vars(s: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/sysconfig.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.sysconfig.expand_makefile_vars(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.sysconfig import expand_makefile_vars
try:
    expand_makefile_vars(12345, None)  # s: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_sysconfig/get_config_var__name_as_Literal_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_sysconfig_get_config_var__name_as_Literal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_sysconfig"
# dimension = "type"
# case = "get_config_var__name_as_Literal_wrong"
# subject = "distutils.sysconfig.get_config_var(name: Literal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/sysconfig.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.sysconfig.get_config_var(name: Literal); call it with the wrong type.

typeshed contract: name is Literal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.sysconfig import get_config_var
try:
    get_config_var(_W())  # name: Literal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_sysconfig/get_config_var__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_sysconfig_get_config_var__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_sysconfig"
# dimension = "type"
# case = "get_config_var__name_as_str_wrong"
# subject = "distutils.sysconfig.get_config_var(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/sysconfig.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.sysconfig.get_config_var(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.sysconfig import get_config_var
try:
    get_config_var(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_sysconfig/get_config_vars__arg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_sysconfig_get_config_vars__arg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_sysconfig"
# dimension = "type"
# case = "get_config_vars__arg_as_str_wrong"
# subject = "distutils.sysconfig.get_config_vars(arg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/sysconfig.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.sysconfig.get_config_vars(arg: str); call it with the wrong type.

typeshed contract: arg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.sysconfig import get_config_vars
try:
    get_config_vars(12345)  # arg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_sysconfig/get_python_inc__plat_specific_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_sysconfig_get_python_inc__plat_specific_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_sysconfig"
# dimension = "type"
# case = "get_python_inc__plat_specific_as_typed_wrong"
# subject = "distutils.sysconfig.get_python_inc(plat_specific: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/sysconfig.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.sysconfig.get_python_inc(plat_specific: typed); call it with the wrong type.

typeshed contract: plat_specific is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.sysconfig import get_python_inc
try:
    get_python_inc(_W())  # plat_specific: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_sysconfig/get_python_lib__plat_specific_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_sysconfig_get_python_lib__plat_specific_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_sysconfig"
# dimension = "type"
# case = "get_python_lib__plat_specific_as_typed_wrong"
# subject = "distutils.sysconfig.get_python_lib(plat_specific: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/sysconfig.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.sysconfig.get_python_lib(plat_specific: typed); call it with the wrong type.

typeshed contract: plat_specific is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.sysconfig import get_python_lib
try:
    get_python_lib(_W())  # plat_specific: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
