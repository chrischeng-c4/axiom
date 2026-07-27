use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/distutils_command_config/config__check_func__func_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_command_config_config__check_func__func_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_command_config"
# dimension = "type"
# case = "config__check_func__func_as_str_wrong"
# subject = "distutils.command.config.config.check_func(func: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/command/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.command.config.config.check_func(func: str); call it with the wrong type.

typeshed contract: func is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.command.config import config
obj = object.__new__(config)
try:
    obj.check_func(12345)  # func: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_command_config/config__check_header__header_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_command_config_config__check_header__header_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_command_config"
# dimension = "type"
# case = "config__check_header__header_as_str_wrong"
# subject = "distutils.command.config.config.check_header(header: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/command/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.command.config.config.check_header(header: str); call it with the wrong type.

typeshed contract: header is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.command.config import config
obj = object.__new__(config)
try:
    obj.check_header(12345)  # header: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_command_config/config__check_lib__library_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_command_config_config__check_lib__library_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_command_config"
# dimension = "type"
# case = "config__check_lib__library_as_str_wrong"
# subject = "distutils.command.config.config.check_lib(library: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/command/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.command.config.config.check_lib(library: str); call it with the wrong type.

typeshed contract: library is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.command.config import config
obj = object.__new__(config)
try:
    obj.check_lib(12345)  # library: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_command_config/config__search_cpp__pattern_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_command_config_config__search_cpp__pattern_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_command_config"
# dimension = "type"
# case = "config__search_cpp__pattern_as_typed_wrong"
# subject = "distutils.command.config.config.search_cpp(pattern: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/command/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.command.config.config.search_cpp(pattern: typed); call it with the wrong type.

typeshed contract: pattern is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.command.config import config
obj = object.__new__(config)
try:
    obj.search_cpp(_W())  # pattern: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_command_config/config__try_compile__body_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_command_config_config__try_compile__body_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_command_config"
# dimension = "type"
# case = "config__try_compile__body_as_str_wrong"
# subject = "distutils.command.config.config.try_compile(body: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/command/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.command.config.config.try_compile(body: str); call it with the wrong type.

typeshed contract: body is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.command.config import config
obj = object.__new__(config)
try:
    obj.try_compile(12345)  # body: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_command_config/config__try_cpp__body_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_command_config_config__try_cpp__body_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_command_config"
# dimension = "type"
# case = "config__try_cpp__body_as_typed_wrong"
# subject = "distutils.command.config.config.try_cpp(body: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/command/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.command.config.config.try_cpp(body: typed); call it with the wrong type.

typeshed contract: body is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.command.config import config
obj = object.__new__(config)
try:
    obj.try_cpp(_W())  # body: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_command_config/config__try_link__body_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_command_config_config__try_link__body_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_command_config"
# dimension = "type"
# case = "config__try_link__body_as_str_wrong"
# subject = "distutils.command.config.config.try_link(body: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/command/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.command.config.config.try_link(body: str); call it with the wrong type.

typeshed contract: body is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.command.config import config
obj = object.__new__(config)
try:
    obj.try_link(12345)  # body: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_command_config/config__try_run__body_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_command_config_config__try_run__body_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_command_config"
# dimension = "type"
# case = "config__try_run__body_as_str_wrong"
# subject = "distutils.command.config.config.try_run(body: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/command/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.command.config.config.try_run(body: str); call it with the wrong type.

typeshed contract: body is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.command.config import config
obj = object.__new__(config)
try:
    obj.try_run(12345)  # body: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_command_config/dump_file__filename_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_command_config_dump_file__filename_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_command_config"
# dimension = "type"
# case = "dump_file__filename_as_StrOrBytesPath_wrong"
# subject = "distutils.command.config.dump_file(filename: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/command/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.command.config.dump_file(filename: StrOrBytesPath); call it with the wrong type.

typeshed contract: filename is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.command.config import dump_file
try:
    dump_file(_W())  # filename: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
