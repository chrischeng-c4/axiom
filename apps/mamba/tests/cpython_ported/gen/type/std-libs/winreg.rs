use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/winreg/CloseKey__hkey_as__KeyType_wrong.py`.
#[test]
fn test_gen_type_std_libs_winreg_CloseKey__hkey_as__KeyType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "CloseKey__hkey_as__KeyType_wrong"
# subject = "winreg.CloseKey(hkey: _KeyType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.CloseKey(hkey: _KeyType); call it with the wrong type.

typeshed contract: hkey is _KeyType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from winreg import CloseKey
try:
    CloseKey(_W())  # hkey: _KeyType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/winreg/ConnectRegistry__computer_name_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_winreg_ConnectRegistry__computer_name_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "ConnectRegistry__computer_name_as_typed_wrong"
# subject = "winreg.ConnectRegistry(computer_name: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.ConnectRegistry(computer_name: typed); call it with the wrong type.

typeshed contract: computer_name is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from winreg import ConnectRegistry
try:
    ConnectRegistry(_W(), None)  # computer_name: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/winreg/CreateKeyEx__key_as__KeyType_wrong.py`.
#[test]
fn test_gen_type_std_libs_winreg_CreateKeyEx__key_as__KeyType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "CreateKeyEx__key_as__KeyType_wrong"
# subject = "winreg.CreateKeyEx(key: _KeyType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.CreateKeyEx(key: _KeyType); call it with the wrong type.

typeshed contract: key is _KeyType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from winreg import CreateKeyEx
try:
    CreateKeyEx(_W(), None)  # key: _KeyType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/winreg/CreateKey__key_as__KeyType_wrong.py`.
#[test]
fn test_gen_type_std_libs_winreg_CreateKey__key_as__KeyType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "CreateKey__key_as__KeyType_wrong"
# subject = "winreg.CreateKey(key: _KeyType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.CreateKey(key: _KeyType); call it with the wrong type.

typeshed contract: key is _KeyType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from winreg import CreateKey
try:
    CreateKey(_W(), None)  # key: _KeyType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/winreg/DeleteKeyEx__key_as__KeyType_wrong.py`.
#[test]
fn test_gen_type_std_libs_winreg_DeleteKeyEx__key_as__KeyType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "DeleteKeyEx__key_as__KeyType_wrong"
# subject = "winreg.DeleteKeyEx(key: _KeyType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.DeleteKeyEx(key: _KeyType); call it with the wrong type.

typeshed contract: key is _KeyType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from winreg import DeleteKeyEx
try:
    DeleteKeyEx(_W(), "")  # key: _KeyType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/winreg/DeleteKey__key_as__KeyType_wrong.py`.
#[test]
fn test_gen_type_std_libs_winreg_DeleteKey__key_as__KeyType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "DeleteKey__key_as__KeyType_wrong"
# subject = "winreg.DeleteKey(key: _KeyType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.DeleteKey(key: _KeyType); call it with the wrong type.

typeshed contract: key is _KeyType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from winreg import DeleteKey
try:
    DeleteKey(_W(), "")  # key: _KeyType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/winreg/DeleteValue__key_as__KeyType_wrong.py`.
#[test]
fn test_gen_type_std_libs_winreg_DeleteValue__key_as__KeyType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "DeleteValue__key_as__KeyType_wrong"
# subject = "winreg.DeleteValue(key: _KeyType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.DeleteValue(key: _KeyType); call it with the wrong type.

typeshed contract: key is _KeyType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from winreg import DeleteValue
try:
    DeleteValue(_W(), "")  # key: _KeyType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/winreg/DisableReflectionKey__key_as__KeyType_wrong.py`.
#[test]
fn test_gen_type_std_libs_winreg_DisableReflectionKey__key_as__KeyType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "DisableReflectionKey__key_as__KeyType_wrong"
# subject = "winreg.DisableReflectionKey(key: _KeyType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.DisableReflectionKey(key: _KeyType); call it with the wrong type.

typeshed contract: key is _KeyType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from winreg import DisableReflectionKey
try:
    DisableReflectionKey(_W())  # key: _KeyType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/winreg/EnableReflectionKey__key_as__KeyType_wrong.py`.
#[test]
fn test_gen_type_std_libs_winreg_EnableReflectionKey__key_as__KeyType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "EnableReflectionKey__key_as__KeyType_wrong"
# subject = "winreg.EnableReflectionKey(key: _KeyType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.EnableReflectionKey(key: _KeyType); call it with the wrong type.

typeshed contract: key is _KeyType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from winreg import EnableReflectionKey
try:
    EnableReflectionKey(_W())  # key: _KeyType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/winreg/EnumKey__key_as__KeyType_wrong.py`.
#[test]
fn test_gen_type_std_libs_winreg_EnumKey__key_as__KeyType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "EnumKey__key_as__KeyType_wrong"
# subject = "winreg.EnumKey(key: _KeyType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.EnumKey(key: _KeyType); call it with the wrong type.

typeshed contract: key is _KeyType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from winreg import EnumKey
try:
    EnumKey(_W(), 0)  # key: _KeyType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/winreg/EnumValue__key_as__KeyType_wrong.py`.
#[test]
fn test_gen_type_std_libs_winreg_EnumValue__key_as__KeyType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "EnumValue__key_as__KeyType_wrong"
# subject = "winreg.EnumValue(key: _KeyType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.EnumValue(key: _KeyType); call it with the wrong type.

typeshed contract: key is _KeyType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from winreg import EnumValue
try:
    EnumValue(_W(), 0)  # key: _KeyType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/winreg/ExpandEnvironmentStrings__string_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_winreg_ExpandEnvironmentStrings__string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "ExpandEnvironmentStrings__string_as_str_wrong"
# subject = "winreg.ExpandEnvironmentStrings(string: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.ExpandEnvironmentStrings(string: str); call it with the wrong type.

typeshed contract: string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from winreg import ExpandEnvironmentStrings
try:
    ExpandEnvironmentStrings(12345)  # string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/winreg/FlushKey__key_as__KeyType_wrong.py`.
#[test]
fn test_gen_type_std_libs_winreg_FlushKey__key_as__KeyType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "FlushKey__key_as__KeyType_wrong"
# subject = "winreg.FlushKey(key: _KeyType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.FlushKey(key: _KeyType); call it with the wrong type.

typeshed contract: key is _KeyType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from winreg import FlushKey
try:
    FlushKey(_W())  # key: _KeyType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/winreg/LoadKey__key_as__KeyType_wrong.py`.
#[test]
fn test_gen_type_std_libs_winreg_LoadKey__key_as__KeyType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "LoadKey__key_as__KeyType_wrong"
# subject = "winreg.LoadKey(key: _KeyType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.LoadKey(key: _KeyType); call it with the wrong type.

typeshed contract: key is _KeyType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from winreg import LoadKey
try:
    LoadKey(_W(), "", "")  # key: _KeyType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/winreg/OpenKeyEx__key_as__KeyType_wrong.py`.
#[test]
fn test_gen_type_std_libs_winreg_OpenKeyEx__key_as__KeyType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "OpenKeyEx__key_as__KeyType_wrong"
# subject = "winreg.OpenKeyEx(key: _KeyType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.OpenKeyEx(key: _KeyType); call it with the wrong type.

typeshed contract: key is _KeyType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from winreg import OpenKeyEx
try:
    OpenKeyEx(_W(), None)  # key: _KeyType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/winreg/OpenKey__key_as__KeyType_wrong.py`.
#[test]
fn test_gen_type_std_libs_winreg_OpenKey__key_as__KeyType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "OpenKey__key_as__KeyType_wrong"
# subject = "winreg.OpenKey(key: _KeyType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.OpenKey(key: _KeyType); call it with the wrong type.

typeshed contract: key is _KeyType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from winreg import OpenKey
try:
    OpenKey(_W(), None)  # key: _KeyType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/winreg/QueryInfoKey__key_as__KeyType_wrong.py`.
#[test]
fn test_gen_type_std_libs_winreg_QueryInfoKey__key_as__KeyType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "QueryInfoKey__key_as__KeyType_wrong"
# subject = "winreg.QueryInfoKey(key: _KeyType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.QueryInfoKey(key: _KeyType); call it with the wrong type.

typeshed contract: key is _KeyType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from winreg import QueryInfoKey
try:
    QueryInfoKey(_W())  # key: _KeyType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/winreg/QueryReflectionKey__key_as__KeyType_wrong.py`.
#[test]
fn test_gen_type_std_libs_winreg_QueryReflectionKey__key_as__KeyType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "QueryReflectionKey__key_as__KeyType_wrong"
# subject = "winreg.QueryReflectionKey(key: _KeyType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.QueryReflectionKey(key: _KeyType); call it with the wrong type.

typeshed contract: key is _KeyType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from winreg import QueryReflectionKey
try:
    QueryReflectionKey(_W())  # key: _KeyType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/winreg/QueryValueEx__key_as__KeyType_wrong.py`.
#[test]
fn test_gen_type_std_libs_winreg_QueryValueEx__key_as__KeyType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "QueryValueEx__key_as__KeyType_wrong"
# subject = "winreg.QueryValueEx(key: _KeyType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.QueryValueEx(key: _KeyType); call it with the wrong type.

typeshed contract: key is _KeyType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from winreg import QueryValueEx
try:
    QueryValueEx(_W(), "")  # key: _KeyType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/winreg/QueryValue__key_as__KeyType_wrong.py`.
#[test]
fn test_gen_type_std_libs_winreg_QueryValue__key_as__KeyType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "QueryValue__key_as__KeyType_wrong"
# subject = "winreg.QueryValue(key: _KeyType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.QueryValue(key: _KeyType); call it with the wrong type.

typeshed contract: key is _KeyType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from winreg import QueryValue
try:
    QueryValue(_W(), None)  # key: _KeyType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/winreg/SaveKey__key_as__KeyType_wrong.py`.
#[test]
fn test_gen_type_std_libs_winreg_SaveKey__key_as__KeyType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "SaveKey__key_as__KeyType_wrong"
# subject = "winreg.SaveKey(key: _KeyType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.SaveKey(key: _KeyType); call it with the wrong type.

typeshed contract: key is _KeyType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from winreg import SaveKey
try:
    SaveKey(_W(), "")  # key: _KeyType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/winreg/SetValue__key_as__KeyType_wrong.py`.
#[test]
fn test_gen_type_std_libs_winreg_SetValue__key_as__KeyType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "SetValue__key_as__KeyType_wrong"
# subject = "winreg.SetValue(key: _KeyType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.SetValue(key: _KeyType); call it with the wrong type.

typeshed contract: key is _KeyType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from winreg import SetValue
try:
    SetValue(_W(), None, 0, "")  # key: _KeyType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
