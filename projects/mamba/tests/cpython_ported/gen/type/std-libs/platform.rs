use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/platform/android_ver__release_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_platform_android_ver__release_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "type"
# case = "android_ver__release_as_str_wrong"
# subject = "platform.android_ver(release: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/platform.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: platform.android_ver(release: str); call it with the wrong type.

typeshed contract: release is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from platform import android_ver
try:
    android_ver(12345)  # release: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/platform/architecture__executable_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_platform_architecture__executable_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "type"
# case = "architecture__executable_as_str_wrong"
# subject = "platform.architecture(executable: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/platform.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: platform.architecture(executable: str); call it with the wrong type.

typeshed contract: executable is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from platform import architecture
try:
    architecture(12345)  # executable: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/platform/ios_ver__system_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_platform_ios_ver__system_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "type"
# case = "ios_ver__system_as_str_wrong"
# subject = "platform.ios_ver(system: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/platform.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: platform.ios_ver(system: str); call it with the wrong type.

typeshed contract: system is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from platform import ios_ver
try:
    ios_ver(12345)  # system: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/platform/java_ver__release_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_platform_java_ver__release_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "type"
# case = "java_ver__release_as_str_wrong"
# subject = "platform.java_ver(release: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/platform.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: platform.java_ver(release: str); call it with the wrong type.

typeshed contract: release is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from platform import java_ver
try:
    java_ver(12345)  # release: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/platform/libc_ver__executable_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_platform_libc_ver__executable_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "type"
# case = "libc_ver__executable_as_typed_wrong"
# subject = "platform.libc_ver(executable: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/platform.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: platform.libc_ver(executable: typed); call it with the wrong type.

typeshed contract: executable is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from platform import libc_ver
try:
    libc_ver(_W())  # executable: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/platform/mac_ver__release_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_platform_mac_ver__release_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "type"
# case = "mac_ver__release_as_str_wrong"
# subject = "platform.mac_ver(release: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/platform.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: platform.mac_ver(release: str); call it with the wrong type.

typeshed contract: release is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from platform import mac_ver
try:
    mac_ver(12345)  # release: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/platform/platform__aliased_as_bool_wrong.py`.
#[test]
fn test_gen_type_std_libs_platform_platform__aliased_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "type"
# case = "platform__aliased_as_bool_wrong"
# subject = "platform.platform(aliased: bool)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/platform.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: platform.platform(aliased: bool); call it with the wrong type.

typeshed contract: aliased is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from platform import platform
try:
    platform("not_a_bool")  # aliased: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/platform/system_alias__system_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_platform_system_alias__system_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "type"
# case = "system_alias__system_as_str_wrong"
# subject = "platform.system_alias(system: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/platform.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: platform.system_alias(system: str); call it with the wrong type.

typeshed contract: system is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from platform import system_alias
try:
    system_alias(12345, "", "")  # system: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/platform/uname_result____new____system_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_platform_uname_result____new____system_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "type"
# case = "uname_result____new____system_as_str_wrong"
# subject = "platform.uname_result.__new__(system: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/platform.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: platform.uname_result.__new__(system: str); call it with the wrong type.

typeshed contract: system is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from platform import uname_result
obj = object.__new__(uname_result)
try:
    obj.__new__(12345, "", "", "", "")  # system: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/platform/win32_ver__release_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_platform_win32_ver__release_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "type"
# case = "win32_ver__release_as_str_wrong"
# subject = "platform.win32_ver(release: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/platform.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: platform.win32_ver(release: str); call it with the wrong type.

typeshed contract: release is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from platform import win32_ver
try:
    win32_ver(12345)  # release: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
