use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/array/array____buffer____flags_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_array_array____buffer____flags_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "type"
# case = "array____buffer____flags_as_int_wrong"
# subject = "array.array.__buffer__(flags: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/array.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: array.array.__buffer__(flags: int); call it with the wrong type.

typeshed contract: flags is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from array import array
obj = object.__new__(array)
try:
    obj.__buffer__("not_an_int")  # flags: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/array/array____imul____value_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_array_array____imul____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "type"
# case = "array____imul____value_as_int_wrong"
# subject = "array.array.__imul__(value: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/array.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: array.array.__imul__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from array import array
obj = object.__new__(array)
try:
    obj.__imul__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/array/array____mul____value_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_array_array____mul____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "type"
# case = "array____mul____value_as_int_wrong"
# subject = "array.array.__mul__(value: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/array.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: array.array.__mul__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from array import array
obj = object.__new__(array)
try:
    obj.__mul__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/array/array____release_buffer____buffer_as_memoryview_wrong.py`.
#[test]
fn test_gen_type_std_libs_array_array____release_buffer____buffer_as_memoryview_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "type"
# case = "array____release_buffer____buffer_as_memoryview_wrong"
# subject = "array.array.__release_buffer__(buffer: memoryview)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/array.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: array.array.__release_buffer__(buffer: memoryview); call it with the wrong type.

typeshed contract: buffer is memoryview. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from array import array
obj = array("i")
try:
    obj.__release_buffer__(12345)  # buffer: memoryview <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/array/array____rmul____value_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_array_array____rmul____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "type"
# case = "array____rmul____value_as_int_wrong"
# subject = "array.array.__rmul__(value: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/array.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: array.array.__rmul__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from array import array
obj = object.__new__(array)
try:
    obj.__rmul__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/array/array__extend__bb_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_array_array__extend__bb_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "type"
# case = "array__extend__bb_as_Iterable_wrong"
# subject = "array.array.extend(bb: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/array.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: array.array.extend(bb: Iterable); call it with the wrong type.

typeshed contract: bb is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from array import array
obj = object.__new__(array)
try:
    obj.extend(_W())  # bb: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/array/array__frombytes__buffer_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_array_array__frombytes__buffer_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "type"
# case = "array__frombytes__buffer_as_ReadableBuffer_wrong"
# subject = "array.array.frombytes(buffer: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/array.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: array.array.frombytes(buffer: ReadableBuffer); call it with the wrong type.

typeshed contract: buffer is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from array import array
obj = object.__new__(array)
try:
    obj.frombytes(_W())  # buffer: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/array/array__fromfile__f_as_SupportsRead_wrong.py`.
#[test]
fn test_gen_type_std_libs_array_array__fromfile__f_as_SupportsRead_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "type"
# case = "array__fromfile__f_as_SupportsRead_wrong"
# subject = "array.array.fromfile(f: SupportsRead)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/array.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: array.array.fromfile(f: SupportsRead); call it with the wrong type.

typeshed contract: f is SupportsRead. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from array import array
obj = object.__new__(array)
try:
    obj.fromfile(_W(), 0)  # f: SupportsRead <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/array/array__fromunicode__ustr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_array_array__fromunicode__ustr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "type"
# case = "array__fromunicode__ustr_as_str_wrong"
# subject = "array.array.fromunicode(ustr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/array.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: array.array.fromunicode(ustr: str); call it with the wrong type.

typeshed contract: ustr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from array import array
obj = object.__new__(array)
try:
    obj.fromunicode(12345)  # ustr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/array/array__insert__i_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_array_array__insert__i_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "type"
# case = "array__insert__i_as_int_wrong"
# subject = "array.array.insert(i: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/array.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: array.array.insert(i: int); call it with the wrong type.

typeshed contract: i is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from array import array
obj = object.__new__(array)
try:
    obj.insert("not_an_int", None)  # i: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/array/array__pop__i_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_array_array__pop__i_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "type"
# case = "array__pop__i_as_int_wrong"
# subject = "array.array.pop(i: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/array.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: array.array.pop(i: int); call it with the wrong type.

typeshed contract: i is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from array import array
obj = object.__new__(array)
try:
    obj.pop("not_an_int")  # i: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/array/array__tofile__f_as_SupportsWrite_wrong.py`.
#[test]
fn test_gen_type_std_libs_array_array__tofile__f_as_SupportsWrite_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "type"
# case = "array__tofile__f_as_SupportsWrite_wrong"
# subject = "array.array.tofile(f: SupportsWrite)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/array.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: array.array.tofile(f: SupportsWrite); call it with the wrong type.

typeshed contract: f is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from array import array
obj = object.__new__(array)
try:
    obj.tofile(_W())  # f: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
