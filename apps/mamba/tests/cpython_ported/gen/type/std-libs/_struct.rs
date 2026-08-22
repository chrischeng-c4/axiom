use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_struct/Struct__init__format_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__struct_Struct__init__format_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_struct"
# dimension = "type"
# case = "Struct__init__format_as_typed_wrong"
# subject = "_struct.Struct.__init__(format: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_struct.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _struct.Struct.__init__(format: typed); call it with the wrong type.

typeshed contract: format is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _struct import Struct
try:
    Struct(_W())  # format: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_struct/Struct__iter_unpack__buffer_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__struct_Struct__iter_unpack__buffer_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_struct"
# dimension = "type"
# case = "Struct__iter_unpack__buffer_as_ReadableBuffer_wrong"
# subject = "_struct.Struct.iter_unpack(buffer: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_struct.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _struct.Struct.iter_unpack(buffer: ReadableBuffer); call it with the wrong type.

typeshed contract: buffer is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _struct import Struct
obj = object.__new__(Struct)
try:
    obj.iter_unpack(_W())  # buffer: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_struct/Struct__pack_into__buffer_as_WriteableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__struct_Struct__pack_into__buffer_as_WriteableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_struct"
# dimension = "type"
# case = "Struct__pack_into__buffer_as_WriteableBuffer_wrong"
# subject = "_struct.Struct.pack_into(buffer: WriteableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_struct.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _struct.Struct.pack_into(buffer: WriteableBuffer); call it with the wrong type.

typeshed contract: buffer is WriteableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _struct import Struct
obj = object.__new__(Struct)
try:
    obj.pack_into(_W(), 0)  # buffer: WriteableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_struct/Struct__unpack__buffer_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__struct_Struct__unpack__buffer_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_struct"
# dimension = "type"
# case = "Struct__unpack__buffer_as_ReadableBuffer_wrong"
# subject = "_struct.Struct.unpack(buffer: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_struct.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _struct.Struct.unpack(buffer: ReadableBuffer); call it with the wrong type.

typeshed contract: buffer is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _struct import Struct
obj = object.__new__(Struct)
try:
    obj.unpack(_W())  # buffer: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_struct/Struct__unpack_from__buffer_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__struct_Struct__unpack_from__buffer_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_struct"
# dimension = "type"
# case = "Struct__unpack_from__buffer_as_ReadableBuffer_wrong"
# subject = "_struct.Struct.unpack_from(buffer: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_struct.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _struct.Struct.unpack_from(buffer: ReadableBuffer); call it with the wrong type.

typeshed contract: buffer is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _struct import Struct
obj = object.__new__(Struct)
try:
    obj.unpack_from(_W())  # buffer: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_struct/calcsize__format_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__struct_calcsize__format_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_struct"
# dimension = "type"
# case = "calcsize__format_as_typed_wrong"
# subject = "_struct.calcsize(format: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_struct.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _struct.calcsize(format: typed); call it with the wrong type.

typeshed contract: format is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _struct import calcsize
try:
    calcsize(_W())  # format: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_struct/iter_unpack__format_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__struct_iter_unpack__format_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_struct"
# dimension = "type"
# case = "iter_unpack__format_as_typed_wrong"
# subject = "_struct.iter_unpack(format: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_struct.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _struct.iter_unpack(format: typed); call it with the wrong type.

typeshed contract: format is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _struct import iter_unpack
try:
    iter_unpack(_W(), None)  # format: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_struct/pack__fmt_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__struct_pack__fmt_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_struct"
# dimension = "type"
# case = "pack__fmt_as_typed_wrong"
# subject = "_struct.pack(fmt: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_struct.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _struct.pack(fmt: typed); call it with the wrong type.

typeshed contract: fmt is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _struct import pack
try:
    pack(_W())  # fmt: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_struct/pack_into__fmt_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__struct_pack_into__fmt_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_struct"
# dimension = "type"
# case = "pack_into__fmt_as_typed_wrong"
# subject = "_struct.pack_into(fmt: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_struct.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _struct.pack_into(fmt: typed); call it with the wrong type.

typeshed contract: fmt is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _struct import pack_into
try:
    pack_into(_W(), None, 0)  # fmt: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_struct/unpack__format_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__struct_unpack__format_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_struct"
# dimension = "type"
# case = "unpack__format_as_typed_wrong"
# subject = "_struct.unpack(format: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_struct.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _struct.unpack(format: typed); call it with the wrong type.

typeshed contract: format is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _struct import unpack
try:
    unpack(_W(), None)  # format: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_struct/unpack_from__format_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__struct_unpack_from__format_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_struct"
# dimension = "type"
# case = "unpack_from__format_as_typed_wrong"
# subject = "_struct.unpack_from(format: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_struct.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _struct.unpack_from(format: typed); call it with the wrong type.

typeshed contract: format is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _struct import unpack_from
try:
    unpack_from(_W(), None)  # format: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
