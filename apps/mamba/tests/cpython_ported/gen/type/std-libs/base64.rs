use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/base64/a85decode__b_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_base64_a85decode__b_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "type"
# case = "a85decode__b_as_typed_wrong"
# subject = "base64.a85decode(b: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/base64.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: base64.a85decode(b: typed); call it with the wrong type.

typeshed contract: b is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from base64 import a85decode
try:
    a85decode(_W())  # b: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/base64/a85encode__b_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_base64_a85encode__b_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "type"
# case = "a85encode__b_as_ReadableBuffer_wrong"
# subject = "base64.a85encode(b: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/base64.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: base64.a85encode(b: ReadableBuffer); call it with the wrong type.

typeshed contract: b is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from base64 import a85encode
try:
    a85encode(_W())  # b: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/base64/b16decode__s_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_base64_b16decode__s_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "type"
# case = "b16decode__s_as_typed_wrong"
# subject = "base64.b16decode(s: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/base64.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: base64.b16decode(s: typed); call it with the wrong type.

typeshed contract: s is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from base64 import b16decode
try:
    b16decode(_W())  # s: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/base64/b16encode__s_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_base64_b16encode__s_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "type"
# case = "b16encode__s_as_ReadableBuffer_wrong"
# subject = "base64.b16encode(s: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/base64.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: base64.b16encode(s: ReadableBuffer); call it with the wrong type.

typeshed contract: s is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from base64 import b16encode
try:
    b16encode(_W())  # s: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/base64/b32decode__s_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_base64_b32decode__s_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "type"
# case = "b32decode__s_as_typed_wrong"
# subject = "base64.b32decode(s: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/base64.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: base64.b32decode(s: typed); call it with the wrong type.

typeshed contract: s is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from base64 import b32decode
try:
    b32decode(_W())  # s: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/base64/b32encode__s_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_base64_b32encode__s_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "type"
# case = "b32encode__s_as_ReadableBuffer_wrong"
# subject = "base64.b32encode(s: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/base64.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: base64.b32encode(s: ReadableBuffer); call it with the wrong type.

typeshed contract: s is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from base64 import b32encode
try:
    b32encode(_W())  # s: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/base64/b32hexdecode__s_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_base64_b32hexdecode__s_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "type"
# case = "b32hexdecode__s_as_typed_wrong"
# subject = "base64.b32hexdecode(s: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/base64.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: base64.b32hexdecode(s: typed); call it with the wrong type.

typeshed contract: s is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from base64 import b32hexdecode
try:
    b32hexdecode(_W())  # s: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/base64/b32hexencode__s_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_base64_b32hexencode__s_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "type"
# case = "b32hexencode__s_as_ReadableBuffer_wrong"
# subject = "base64.b32hexencode(s: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/base64.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: base64.b32hexencode(s: ReadableBuffer); call it with the wrong type.

typeshed contract: s is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from base64 import b32hexencode
try:
    b32hexencode(_W())  # s: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/base64/b64decode__s_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_base64_b64decode__s_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "type"
# case = "b64decode__s_as_typed_wrong"
# subject = "base64.b64decode(s: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/base64.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: base64.b64decode(s: typed); call it with the wrong type.

typeshed contract: s is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from base64 import b64decode
try:
    b64decode(_W())  # s: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/base64/b64encode__s_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_base64_b64encode__s_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "type"
# case = "b64encode__s_as_ReadableBuffer_wrong"
# subject = "base64.b64encode(s: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/base64.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: base64.b64encode(s: ReadableBuffer); call it with the wrong type.

typeshed contract: s is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from base64 import b64encode
try:
    b64encode(_W())  # s: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/base64/b85decode__b_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_base64_b85decode__b_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "type"
# case = "b85decode__b_as_typed_wrong"
# subject = "base64.b85decode(b: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/base64.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: base64.b85decode(b: typed); call it with the wrong type.

typeshed contract: b is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from base64 import b85decode
try:
    b85decode(_W())  # b: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/base64/b85encode__b_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_base64_b85encode__b_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "type"
# case = "b85encode__b_as_ReadableBuffer_wrong"
# subject = "base64.b85encode(b: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/base64.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: base64.b85encode(b: ReadableBuffer); call it with the wrong type.

typeshed contract: b is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from base64 import b85encode
try:
    b85encode(_W())  # b: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/base64/decode__input_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_base64_decode__input_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "type"
# case = "decode__input_as_IO_wrong"
# subject = "base64.decode(input: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/base64.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: base64.decode(input: IO); call it with the wrong type.

typeshed contract: input is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from base64 import decode
try:
    decode(_W(), None)  # input: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/base64/decodebytes__s_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_base64_decodebytes__s_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "type"
# case = "decodebytes__s_as_ReadableBuffer_wrong"
# subject = "base64.decodebytes(s: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/base64.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: base64.decodebytes(s: ReadableBuffer); call it with the wrong type.

typeshed contract: s is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from base64 import decodebytes
try:
    decodebytes(_W())  # s: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/base64/encode__input_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_base64_encode__input_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "type"
# case = "encode__input_as_IO_wrong"
# subject = "base64.encode(input: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/base64.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: base64.encode(input: IO); call it with the wrong type.

typeshed contract: input is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from base64 import encode
try:
    encode(_W(), None)  # input: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/base64/encodebytes__s_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_base64_encodebytes__s_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "type"
# case = "encodebytes__s_as_ReadableBuffer_wrong"
# subject = "base64.encodebytes(s: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/base64.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: base64.encodebytes(s: ReadableBuffer); call it with the wrong type.

typeshed contract: s is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from base64 import encodebytes
try:
    encodebytes(_W())  # s: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/base64/standard_b64decode__s_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_base64_standard_b64decode__s_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "type"
# case = "standard_b64decode__s_as_typed_wrong"
# subject = "base64.standard_b64decode(s: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/base64.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: base64.standard_b64decode(s: typed); call it with the wrong type.

typeshed contract: s is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from base64 import standard_b64decode
try:
    standard_b64decode(_W())  # s: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/base64/standard_b64encode__s_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_base64_standard_b64encode__s_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "type"
# case = "standard_b64encode__s_as_ReadableBuffer_wrong"
# subject = "base64.standard_b64encode(s: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/base64.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: base64.standard_b64encode(s: ReadableBuffer); call it with the wrong type.

typeshed contract: s is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from base64 import standard_b64encode
try:
    standard_b64encode(_W())  # s: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/base64/urlsafe_b64decode__s_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_base64_urlsafe_b64decode__s_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "type"
# case = "urlsafe_b64decode__s_as_typed_wrong"
# subject = "base64.urlsafe_b64decode(s: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/base64.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: base64.urlsafe_b64decode(s: typed); call it with the wrong type.

typeshed contract: s is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from base64 import urlsafe_b64decode
try:
    urlsafe_b64decode(_W())  # s: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/base64/urlsafe_b64encode__s_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_base64_urlsafe_b64encode__s_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "type"
# case = "urlsafe_b64encode__s_as_ReadableBuffer_wrong"
# subject = "base64.urlsafe_b64encode(s: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/base64.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: base64.urlsafe_b64encode(s: ReadableBuffer); call it with the wrong type.

typeshed contract: s is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from base64 import urlsafe_b64encode
try:
    urlsafe_b64encode(_W())  # s: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
