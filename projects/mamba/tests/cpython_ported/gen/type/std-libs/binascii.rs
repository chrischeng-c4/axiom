use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/binascii/a2b_ascii85__data_as__AsciiBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_binascii_a2b_ascii85__data_as__AsciiBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "type"
# case = "a2b_ascii85__data_as__AsciiBuffer_wrong"
# subject = "binascii.a2b_ascii85(data: _AsciiBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binascii.a2b_ascii85(data: _AsciiBuffer); call it with the wrong type.

typeshed contract: data is _AsciiBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binascii import a2b_ascii85
try:
    a2b_ascii85(_W())  # data: _AsciiBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binascii/a2b_base32__data_as__AsciiBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_binascii_a2b_base32__data_as__AsciiBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "type"
# case = "a2b_base32__data_as__AsciiBuffer_wrong"
# subject = "binascii.a2b_base32(data: _AsciiBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binascii.a2b_base32(data: _AsciiBuffer); call it with the wrong type.

typeshed contract: data is _AsciiBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binascii import a2b_base32
try:
    a2b_base32(_W())  # data: _AsciiBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binascii/a2b_base64__data_as__AsciiBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_binascii_a2b_base64__data_as__AsciiBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "type"
# case = "a2b_base64__data_as__AsciiBuffer_wrong"
# subject = "binascii.a2b_base64(data: _AsciiBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binascii.a2b_base64(data: _AsciiBuffer); call it with the wrong type.

typeshed contract: data is _AsciiBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binascii import a2b_base64
try:
    a2b_base64(_W())  # data: _AsciiBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binascii/a2b_base85__data_as__AsciiBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_binascii_a2b_base85__data_as__AsciiBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "type"
# case = "a2b_base85__data_as__AsciiBuffer_wrong"
# subject = "binascii.a2b_base85(data: _AsciiBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binascii.a2b_base85(data: _AsciiBuffer); call it with the wrong type.

typeshed contract: data is _AsciiBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binascii import a2b_base85
try:
    a2b_base85(_W())  # data: _AsciiBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binascii/a2b_hex__hexstr_as__AsciiBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_binascii_a2b_hex__hexstr_as__AsciiBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "type"
# case = "a2b_hex__hexstr_as__AsciiBuffer_wrong"
# subject = "binascii.a2b_hex(hexstr: _AsciiBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binascii.a2b_hex(hexstr: _AsciiBuffer); call it with the wrong type.

typeshed contract: hexstr is _AsciiBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binascii import a2b_hex
try:
    a2b_hex(_W())  # hexstr: _AsciiBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binascii/a2b_hqx__data_as__AsciiBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_binascii_a2b_hqx__data_as__AsciiBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "type"
# case = "a2b_hqx__data_as__AsciiBuffer_wrong"
# subject = "binascii.a2b_hqx(data: _AsciiBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binascii.a2b_hqx(data: _AsciiBuffer); call it with the wrong type.

typeshed contract: data is _AsciiBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binascii import a2b_hqx
try:
    a2b_hqx(_W())  # data: _AsciiBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binascii/a2b_qp__data_as__AsciiBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_binascii_a2b_qp__data_as__AsciiBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "type"
# case = "a2b_qp__data_as__AsciiBuffer_wrong"
# subject = "binascii.a2b_qp(data: _AsciiBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binascii.a2b_qp(data: _AsciiBuffer); call it with the wrong type.

typeshed contract: data is _AsciiBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binascii import a2b_qp
try:
    a2b_qp(_W())  # data: _AsciiBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binascii/a2b_uu__data_as__AsciiBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_binascii_a2b_uu__data_as__AsciiBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "type"
# case = "a2b_uu__data_as__AsciiBuffer_wrong"
# subject = "binascii.a2b_uu(data: _AsciiBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binascii.a2b_uu(data: _AsciiBuffer); call it with the wrong type.

typeshed contract: data is _AsciiBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binascii import a2b_uu
try:
    a2b_uu(_W())  # data: _AsciiBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binascii/b2a_ascii85__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_binascii_b2a_ascii85__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "type"
# case = "b2a_ascii85__data_as_ReadableBuffer_wrong"
# subject = "binascii.b2a_ascii85(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binascii.b2a_ascii85(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binascii import b2a_ascii85
try:
    b2a_ascii85(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binascii/b2a_base32__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_binascii_b2a_base32__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "type"
# case = "b2a_base32__data_as_ReadableBuffer_wrong"
# subject = "binascii.b2a_base32(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binascii.b2a_base32(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binascii import b2a_base32
try:
    b2a_base32(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binascii/b2a_base64__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_binascii_b2a_base64__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "type"
# case = "b2a_base64__data_as_ReadableBuffer_wrong"
# subject = "binascii.b2a_base64(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binascii.b2a_base64(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binascii import b2a_base64
try:
    b2a_base64(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binascii/b2a_base85__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_binascii_b2a_base85__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "type"
# case = "b2a_base85__data_as_ReadableBuffer_wrong"
# subject = "binascii.b2a_base85(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binascii.b2a_base85(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binascii import b2a_base85
try:
    b2a_base85(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binascii/b2a_hex__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_binascii_b2a_hex__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "type"
# case = "b2a_hex__data_as_ReadableBuffer_wrong"
# subject = "binascii.b2a_hex(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binascii.b2a_hex(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binascii import b2a_hex
try:
    b2a_hex(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binascii/b2a_hqx__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_binascii_b2a_hqx__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "type"
# case = "b2a_hqx__data_as_ReadableBuffer_wrong"
# subject = "binascii.b2a_hqx(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binascii.b2a_hqx(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binascii import b2a_hqx
try:
    b2a_hqx(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binascii/b2a_qp__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_binascii_b2a_qp__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "type"
# case = "b2a_qp__data_as_ReadableBuffer_wrong"
# subject = "binascii.b2a_qp(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binascii.b2a_qp(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binascii import b2a_qp
try:
    b2a_qp(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binascii/b2a_uu__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_binascii_b2a_uu__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "type"
# case = "b2a_uu__data_as_ReadableBuffer_wrong"
# subject = "binascii.b2a_uu(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binascii.b2a_uu(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binascii import b2a_uu
try:
    b2a_uu(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binascii/crc32__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_binascii_crc32__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "type"
# case = "crc32__data_as_ReadableBuffer_wrong"
# subject = "binascii.crc32(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binascii.crc32(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binascii import crc32
try:
    crc32(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binascii/crc_hqx__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_binascii_crc_hqx__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "type"
# case = "crc_hqx__data_as_ReadableBuffer_wrong"
# subject = "binascii.crc_hqx(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binascii.crc_hqx(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binascii import crc_hqx
try:
    crc_hqx(_W(), 0)  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binascii/hexlify__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_binascii_hexlify__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "type"
# case = "hexlify__data_as_ReadableBuffer_wrong"
# subject = "binascii.hexlify(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binascii.hexlify(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binascii import hexlify
try:
    hexlify(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binascii/rlecode_hqx__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_binascii_rlecode_hqx__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "type"
# case = "rlecode_hqx__data_as_ReadableBuffer_wrong"
# subject = "binascii.rlecode_hqx(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binascii.rlecode_hqx(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binascii import rlecode_hqx
try:
    rlecode_hqx(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binascii/rledecode_hqx__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_binascii_rledecode_hqx__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "type"
# case = "rledecode_hqx__data_as_ReadableBuffer_wrong"
# subject = "binascii.rledecode_hqx(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binascii.rledecode_hqx(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binascii import rledecode_hqx
try:
    rledecode_hqx(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binascii/unhexlify__hexstr_as__AsciiBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_binascii_unhexlify__hexstr_as__AsciiBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "type"
# case = "unhexlify__hexstr_as__AsciiBuffer_wrong"
# subject = "binascii.unhexlify(hexstr: _AsciiBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binascii.unhexlify(hexstr: _AsciiBuffer); call it with the wrong type.

typeshed contract: hexstr is _AsciiBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binascii import unhexlify
try:
    unhexlify(_W())  # hexstr: _AsciiBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
