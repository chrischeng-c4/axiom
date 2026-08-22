use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_ctypes/Array____getitem____key_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_Array____getitem____key_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "Array____getitem____key_as_int_wrong"
# subject = "_ctypes.Array.__getitem__(key: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.Array.__getitem__(key: int); call it with the wrong type.

typeshed contract: key is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

import _ctypes
import ctypes

obj = (ctypes.c_int * 2)(1, 2)
try:
    _ctypes.Array.__getitem__(obj, "not_an_int")  # key: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/Array____getitem____key_as_slice_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_Array____getitem____key_as_slice_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "Array____getitem____key_as_slice_wrong"
# subject = "_ctypes.Array.__getitem__(key: slice)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.Array.__getitem__(key: slice); call it with the wrong type.

typeshed contract: key is slice. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


import _ctypes
import ctypes

obj = (ctypes.c_int * 2)(1, 2)
try:
    _ctypes.Array.__getitem__(obj, _W())  # key: slice <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/Array____setitem____key_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_Array____setitem____key_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "Array____setitem____key_as_int_wrong"
# subject = "_ctypes.Array.__setitem__(key: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.Array.__setitem__(key: int); call it with the wrong type.

typeshed contract: key is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

import _ctypes
import ctypes

obj = (ctypes.c_int * 2)(1, 2)
try:
    _ctypes.Array.__setitem__(obj, "not_an_int", 1)  # key: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/Array____setitem____key_as_slice_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_Array____setitem____key_as_slice_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "Array____setitem____key_as_slice_wrong"
# subject = "_ctypes.Array.__setitem__(key: slice)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.Array.__setitem__(key: slice); call it with the wrong type.

typeshed contract: key is slice. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


import _ctypes
import ctypes

obj = (ctypes.c_int * 2)(1, 2)
try:
    _ctypes.Array.__setitem__(obj, _W(), 1)  # key: slice <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/CFuncPtr____new____address_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_CFuncPtr____new____address_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "CFuncPtr____new____address_as_int_wrong"
# subject = "_ctypes.CFuncPtr.__new__(address: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.CFuncPtr.__new__(address: int); call it with the wrong type.

typeshed contract: address is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ctypes import CFuncPtr
try:
    CFuncPtr.__new__(CFuncPtr, "not_an_int")  # address: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/CFuncPtr____new____callable_as_Callable_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_CFuncPtr____new____callable_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "CFuncPtr____new____callable_as_Callable_wrong"
# subject = "_ctypes.CFuncPtr.__new__(callable: Callable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.CFuncPtr.__new__(callable: Callable); call it with the wrong type.

typeshed contract: callable is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _ctypes import CFuncPtr
try:
    CFuncPtr.__new__(CFuncPtr, _W())  # callable: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/COMError__init__hresult_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_COMError__init__hresult_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "COMError__init__hresult_as_int_wrong"
# subject = "_ctypes.COMError.__init__(hresult: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.COMError.__init__(hresult: int); call it with the wrong type.

typeshed contract: hresult is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ctypes import COMError
try:
    COMError("not_an_int", None, None)  # hresult: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/CopyComPointer__src_as__PointerLike_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_CopyComPointer__src_as__PointerLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "CopyComPointer__src_as__PointerLike_wrong"
# subject = "_ctypes.CopyComPointer(src: _PointerLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.CopyComPointer(src: _PointerLike); call it with the wrong type.

typeshed contract: src is _PointerLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _ctypes import CopyComPointer
try:
    CopyComPointer(_W(), None)  # src: _PointerLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/FormatError__code_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_FormatError__code_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "FormatError__code_as_int_wrong"
# subject = "_ctypes.FormatError(code: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.FormatError(code: int); call it with the wrong type.

typeshed contract: code is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ctypes import FormatError
try:
    FormatError("not_an_int")  # code: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/FreeLibrary__handle_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_FreeLibrary__handle_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "FreeLibrary__handle_as_int_wrong"
# subject = "_ctypes.FreeLibrary(handle: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.FreeLibrary(handle: int); call it with the wrong type.

typeshed contract: handle is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ctypes import FreeLibrary
try:
    FreeLibrary("not_an_int")  # handle: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/LoadLibrary__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_LoadLibrary__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "LoadLibrary__name_as_str_wrong"
# subject = "_ctypes.LoadLibrary(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.LoadLibrary(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ctypes import LoadLibrary
try:
    LoadLibrary(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/POINTER__type_as_type_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_POINTER__type_as_type_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "POINTER__type_as_type_wrong"
# subject = "_ctypes.POINTER(type: type)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.POINTER(type: type); call it with the wrong type.

typeshed contract: type is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _ctypes import POINTER
try:
    POINTER(_W())  # type: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/POINTER__type_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_POINTER__type_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "POINTER__type_as_typed_wrong"
# subject = "_ctypes.POINTER(type: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.POINTER(type: typed); call it with the wrong type.

typeshed contract: type is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _ctypes import POINTER
try:
    POINTER(_W())  # type: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/PyObj_FromPtr__address_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_PyObj_FromPtr__address_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "PyObj_FromPtr__address_as_int_wrong"
# subject = "_ctypes.PyObj_FromPtr(address: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.PyObj_FromPtr(address: int); call it with the wrong type.

typeshed contract: address is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ctypes import PyObj_FromPtr
try:
    PyObj_FromPtr("not_an_int")  # address: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/Structure____getattr____name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_Structure____getattr____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "Structure____getattr____name_as_str_wrong"
# subject = "_ctypes.Structure.__getattr__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.Structure.__getattr__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ctypes import Structure
obj = object.__new__(Structure)
try:
    obj.__getattr__(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/Structure____setattr____name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_Structure____setattr____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "Structure____setattr____name_as_str_wrong"
# subject = "_ctypes.Structure.__setattr__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.Structure.__setattr__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ctypes import Structure
obj = object.__new__(Structure)
try:
    obj.__setattr__(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/Union____getattr____name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_Union____getattr____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "Union____getattr____name_as_str_wrong"
# subject = "_ctypes.Union.__getattr__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.Union.__getattr__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ctypes import Union
obj = object.__new__(Union)
try:
    obj.__getattr__(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/Union____setattr____name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_Union____setattr____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "Union____setattr____name_as_str_wrong"
# subject = "_ctypes.Union.__setattr__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.Union.__setattr__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ctypes import Union
obj = object.__new__(Union)
try:
    obj.__setattr__(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/addressof__obj_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_addressof__obj_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "addressof__obj_as_typed_wrong"
# subject = "_ctypes.addressof(obj: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.addressof(obj: typed); call it with the wrong type.

typeshed contract: obj is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _ctypes import addressof
try:
    addressof(_W())  # obj: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/alignment__obj_or_type_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_alignment__obj_or_type_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "alignment__obj_or_type_as_typed_wrong"
# subject = "_ctypes.alignment(obj_or_type: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.alignment(obj_or_type: typed); call it with the wrong type.

typeshed contract: obj_or_type is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _ctypes import alignment
try:
    alignment(_W())  # obj_or_type: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/buffer_info__o_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_buffer_info__o_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "buffer_info__o_as_typed_wrong"
# subject = "_ctypes.buffer_info(o: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.buffer_info(o: typed); call it with the wrong type.

typeshed contract: o is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _ctypes import buffer_info
try:
    buffer_info(_W())  # o: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/byref__obj_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_byref__obj_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "byref__obj_as_typed_wrong"
# subject = "_ctypes.byref(obj: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.byref(obj: typed); call it with the wrong type.

typeshed contract: obj is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _ctypes import byref
try:
    byref(_W())  # obj: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/call_cdeclfunction__address_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_call_cdeclfunction__address_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "call_cdeclfunction__address_as_int_wrong"
# subject = "_ctypes.call_cdeclfunction(address: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.call_cdeclfunction(address: int); call it with the wrong type.

typeshed contract: address is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ctypes import call_cdeclfunction
try:
    call_cdeclfunction("not_an_int", None)  # address: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/call_function__address_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_call_function__address_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "call_function__address_as_int_wrong"
# subject = "_ctypes.call_function(address: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.call_function(address: int); call it with the wrong type.

typeshed contract: address is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ctypes import call_function
try:
    call_function("not_an_int", None)  # address: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/dlclose__handle_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_dlclose__handle_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "dlclose__handle_as_int_wrong"
# subject = "_ctypes.dlclose(handle: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.dlclose(handle: int); call it with the wrong type.

typeshed contract: handle is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ctypes import dlclose
try:
    dlclose("not_an_int")  # handle: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/dlopen__name_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_dlopen__name_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "dlopen__name_as_StrOrBytesPath_wrong"
# subject = "_ctypes.dlopen(name: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.dlopen(name: StrOrBytesPath); call it with the wrong type.

typeshed contract: name is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _ctypes import dlopen
try:
    dlopen(_W())  # name: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/dlsym__handle_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_dlsym__handle_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "dlsym__handle_as_int_wrong"
# subject = "_ctypes.dlsym(handle: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.dlsym(handle: int); call it with the wrong type.

typeshed contract: handle is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ctypes import dlsym
try:
    dlsym("not_an_int", "")  # handle: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/pointer__obj_as__CT_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_pointer__obj_as__CT_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "pointer__obj_as__CT_wrong"
# subject = "_ctypes.pointer(obj: _CT)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.pointer(obj: _CT); call it with the wrong type.

typeshed contract: obj is _CT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _ctypes import pointer
try:
    pointer(_W())  # obj: _CT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/resize__obj_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_resize__obj_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "resize__obj_as_typed_wrong"
# subject = "_ctypes.resize(obj: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.resize(obj: typed); call it with the wrong type.

typeshed contract: obj is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _ctypes import resize
try:
    resize(_W(), 0)  # obj: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/set_errno__value_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_set_errno__value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "set_errno__value_as_int_wrong"
# subject = "_ctypes.set_errno(value: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.set_errno(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ctypes import set_errno
try:
    set_errno("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/set_last_error__value_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_set_last_error__value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "set_last_error__value_as_int_wrong"
# subject = "_ctypes.set_last_error(value: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.set_last_error(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ctypes import set_last_error
try:
    set_last_error("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ctypes/sizeof__obj_or_type_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__ctypes_sizeof__obj_or_type_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "sizeof__obj_or_type_as_typed_wrong"
# subject = "_ctypes.sizeof(obj_or_type: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.sizeof(obj_or_type: typed); call it with the wrong type.

typeshed contract: obj_or_type is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _ctypes import sizeof
try:
    sizeof(_W())  # obj_or_type: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ctypes/CDLL____getattr____name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ctypes_CDLL____getattr____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "type"
# case = "CDLL____getattr____name_as_str_wrong"
# subject = "ctypes.CDLL.__getattr__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ctypes.CDLL.__getattr__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ctypes import CDLL
obj = object.__new__(CDLL)
try:
    obj.__getattr__(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ctypes/CDLL____getitem____name_or_ordinal_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ctypes_CDLL____getitem____name_or_ordinal_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "type"
# case = "CDLL____getitem____name_or_ordinal_as_str_wrong"
# subject = "ctypes.CDLL.__getitem__(name_or_ordinal: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ctypes.CDLL.__getitem__(name_or_ordinal: str); call it with the wrong type.

typeshed contract: name_or_ordinal is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ctypes import CDLL
obj = object.__new__(CDLL)
try:
    obj.__getitem__(12345)  # name_or_ordinal: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ctypes/CDLL__init__name_as__NameTypes_wrong.py`.
#[test]
fn test_gen_type_std_libs_ctypes_CDLL__init__name_as__NameTypes_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "type"
# case = "CDLL__init__name_as__NameTypes_wrong"
# subject = "ctypes.CDLL.__init__(name: _NameTypes)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ctypes.CDLL.__init__(name: _NameTypes); call it with the wrong type.

typeshed contract: name is _NameTypes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ctypes import CDLL
try:
    CDLL(_W())  # name: _NameTypes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ctypes/LibraryLoader__LoadLibrary__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ctypes_LibraryLoader__LoadLibrary__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "type"
# case = "LibraryLoader__LoadLibrary__name_as_str_wrong"
# subject = "ctypes.LibraryLoader.LoadLibrary(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ctypes.LibraryLoader.LoadLibrary(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ctypes import LibraryLoader
obj = object.__new__(LibraryLoader)
try:
    obj.LoadLibrary(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ctypes/LibraryLoader____getattr____name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ctypes_LibraryLoader____getattr____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "type"
# case = "LibraryLoader____getattr____name_as_str_wrong"
# subject = "ctypes.LibraryLoader.__getattr__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ctypes.LibraryLoader.__getattr__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ctypes import LibraryLoader
obj = object.__new__(LibraryLoader)
try:
    obj.__getattr__(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ctypes/LibraryLoader____getitem____name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ctypes_LibraryLoader____getitem____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "type"
# case = "LibraryLoader____getitem____name_as_str_wrong"
# subject = "ctypes.LibraryLoader.__getitem__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ctypes.LibraryLoader.__getitem__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ctypes import LibraryLoader
obj = object.__new__(LibraryLoader)
try:
    obj.__getitem__(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ctypes/WinError__code_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_ctypes_WinError__code_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "type"
# case = "WinError__code_as_typed_wrong"
# subject = "ctypes.WinError(code: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ctypes.WinError(code: typed); call it with the wrong type.

typeshed contract: code is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ctypes import WinError
try:
    WinError(_W())  # code: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ctypes/c_bool__init__value_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_ctypes_c_bool__init__value_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "type"
# case = "c_bool__init__value_as_typed_wrong"
# subject = "ctypes.c_bool.__init__(value: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ctypes.c_bool.__init__(value: typed); call it with the wrong type.

typeshed contract: value is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ctypes import c_bool
try:
    c_bool(_W())  # value: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ctypes/c_char__init__value_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_ctypes_c_char__init__value_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "type"
# case = "c_char__init__value_as_typed_wrong"
# subject = "ctypes.c_char.__init__(value: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ctypes.c_char.__init__(value: typed); call it with the wrong type.

typeshed contract: value is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ctypes import c_char
try:
    c_char(_W())  # value: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ctypes/c_char_p__init__value_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_ctypes_c_char_p__init__value_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "type"
# case = "c_char_p__init__value_as_typed_wrong"
# subject = "ctypes.c_char_p.__init__(value: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ctypes.c_char_p.__init__(value: typed); call it with the wrong type.

typeshed contract: value is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ctypes import c_char_p
try:
    c_char_p(_W())  # value: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ctypes/c_wchar_p__init__value_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_ctypes_c_wchar_p__init__value_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "type"
# case = "c_wchar_p__init__value_as_typed_wrong"
# subject = "ctypes.c_wchar_p.__init__(value: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ctypes.c_wchar_p.__init__(value: typed); call it with the wrong type.

typeshed contract: value is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ctypes import c_wchar_p
try:
    c_wchar_p(_W())  # value: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ctypes/cast__obj_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_ctypes_cast__obj_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "type"
# case = "cast__obj_as_typed_wrong"
# subject = "ctypes.cast(obj: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ctypes.cast(obj: typed); call it with the wrong type.

typeshed contract: obj is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ctypes import cast
try:
    cast(_W(), None)  # obj: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ctypes/create_string_buffer__init_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_ctypes_create_string_buffer__init_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "type"
# case = "create_string_buffer__init_as_typed_wrong"
# subject = "ctypes.create_string_buffer(init: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ctypes.create_string_buffer(init: typed); call it with the wrong type.

typeshed contract: init is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ctypes import create_string_buffer
try:
    create_string_buffer(_W())  # init: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ctypes/create_unicode_buffer__init_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_ctypes_create_unicode_buffer__init_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "type"
# case = "create_unicode_buffer__init_as_typed_wrong"
# subject = "ctypes.create_unicode_buffer(init: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ctypes.create_unicode_buffer(init: typed); call it with the wrong type.

typeshed contract: init is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ctypes import create_unicode_buffer
try:
    create_unicode_buffer(_W())  # init: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ctypes/memoryview_at__ptr_as__CVoidConstPLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_ctypes_memoryview_at__ptr_as__CVoidConstPLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "type"
# case = "memoryview_at__ptr_as__CVoidConstPLike_wrong"
# subject = "ctypes.memoryview_at(ptr: _CVoidConstPLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ctypes.memoryview_at(ptr: _CVoidConstPLike); call it with the wrong type.

typeshed contract: ptr is _CVoidConstPLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ctypes import memoryview_at
try:
    memoryview_at(_W(), 0)  # ptr: _CVoidConstPLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ctypes/string_at__ptr_as__CVoidConstPLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_ctypes_string_at__ptr_as__CVoidConstPLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "type"
# case = "string_at__ptr_as__CVoidConstPLike_wrong"
# subject = "ctypes.string_at(ptr: _CVoidConstPLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ctypes.string_at(ptr: _CVoidConstPLike); call it with the wrong type.

typeshed contract: ptr is _CVoidConstPLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ctypes import string_at
try:
    string_at(_W())  # ptr: _CVoidConstPLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ctypes/wstring_at__ptr_as__CVoidConstPLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_ctypes_wstring_at__ptr_as__CVoidConstPLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "type"
# case = "wstring_at__ptr_as__CVoidConstPLike_wrong"
# subject = "ctypes.wstring_at(ptr: _CVoidConstPLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ctypes.wstring_at(ptr: _CVoidConstPLike); call it with the wrong type.

typeshed contract: ptr is _CVoidConstPLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ctypes import wstring_at
try:
    wstring_at(_W())  # ptr: _CVoidConstPLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
