use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_hashlib/HASHXOF__digest__length_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__hashlib_HASHXOF__digest__length_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "HASHXOF__digest__length_as_int_wrong"
# subject = "_hashlib.HASHXOF.digest(length: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.HASHXOF.digest(length: int); call it with the wrong type.

typeshed contract: length is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _hashlib import HASHXOF
obj = object.__new__(HASHXOF)
try:
    obj.digest("not_an_int")  # length: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_hashlib/HASHXOF__hexdigest__length_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__hashlib_HASHXOF__hexdigest__length_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "HASHXOF__hexdigest__length_as_int_wrong"
# subject = "_hashlib.HASHXOF.hexdigest(length: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.HASHXOF.hexdigest(length: int); call it with the wrong type.

typeshed contract: length is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _hashlib import HASHXOF
obj = object.__new__(HASHXOF)
try:
    obj.hexdigest("not_an_int")  # length: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_hashlib/HASH__update__obj_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__hashlib_HASH__update__obj_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "HASH__update__obj_as_ReadableBuffer_wrong"
# subject = "_hashlib.HASH.update(obj: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.HASH.update(obj: ReadableBuffer); call it with the wrong type.

typeshed contract: obj is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _hashlib import HASH
obj = object.__new__(HASH)
try:
    obj.update(_W())  # obj: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_hashlib/HMAC__update__msg_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__hashlib_HMAC__update__msg_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "HMAC__update__msg_as_ReadableBuffer_wrong"
# subject = "_hashlib.HMAC.update(msg: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.HMAC.update(msg: ReadableBuffer); call it with the wrong type.

typeshed contract: msg is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _hashlib import HMAC
obj = object.__new__(HMAC)
try:
    obj.update(_W())  # msg: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_hashlib/compare_digest__a_as_AnyStr_wrong.py`.
#[test]
fn test_gen_type_std_libs__hashlib_compare_digest__a_as_AnyStr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "compare_digest__a_as_AnyStr_wrong"
# subject = "_hashlib.compare_digest(a: AnyStr)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.compare_digest(a: AnyStr); call it with the wrong type.

typeshed contract: a is AnyStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _hashlib import compare_digest
try:
    compare_digest(_W(), None)  # a: AnyStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_hashlib/compare_digest__a_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__hashlib_compare_digest__a_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "compare_digest__a_as_ReadableBuffer_wrong"
# subject = "_hashlib.compare_digest(a: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.compare_digest(a: ReadableBuffer); call it with the wrong type.

typeshed contract: a is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _hashlib import compare_digest
try:
    compare_digest(_W(), None)  # a: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_hashlib/hmac_digest__key_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__hashlib_hmac_digest__key_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "hmac_digest__key_as_ReadableBuffer_wrong"
# subject = "_hashlib.hmac_digest(key: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.hmac_digest(key: ReadableBuffer); call it with the wrong type.

typeshed contract: key is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _hashlib import hmac_digest
try:
    hmac_digest(_W(), None, "")  # key: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_hashlib/hmac_new__key_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__hashlib_hmac_new__key_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "hmac_new__key_as_ReadableBuffer_wrong"
# subject = "_hashlib.hmac_new(key: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.hmac_new(key: ReadableBuffer); call it with the wrong type.

typeshed contract: key is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _hashlib import hmac_new
try:
    hmac_new(_W())  # key: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_hashlib/new__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__hashlib_new__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "new__name_as_str_wrong"
# subject = "_hashlib.new(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.new(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _hashlib import new
try:
    new(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_hashlib/openssl_md5__string_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__hashlib_openssl_md5__string_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "openssl_md5__string_as_ReadableBuffer_wrong"
# subject = "_hashlib.openssl_md5(string: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.openssl_md5(string: ReadableBuffer); call it with the wrong type.

typeshed contract: string is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _hashlib import openssl_md5
try:
    openssl_md5(_W())  # string: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_hashlib/openssl_sha1__string_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__hashlib_openssl_sha1__string_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "openssl_sha1__string_as_ReadableBuffer_wrong"
# subject = "_hashlib.openssl_sha1(string: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.openssl_sha1(string: ReadableBuffer); call it with the wrong type.

typeshed contract: string is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _hashlib import openssl_sha1
try:
    openssl_sha1(_W())  # string: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_hashlib/openssl_sha224__string_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__hashlib_openssl_sha224__string_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "openssl_sha224__string_as_ReadableBuffer_wrong"
# subject = "_hashlib.openssl_sha224(string: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.openssl_sha224(string: ReadableBuffer); call it with the wrong type.

typeshed contract: string is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _hashlib import openssl_sha224
try:
    openssl_sha224(_W())  # string: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_hashlib/openssl_sha256__string_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__hashlib_openssl_sha256__string_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "openssl_sha256__string_as_ReadableBuffer_wrong"
# subject = "_hashlib.openssl_sha256(string: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.openssl_sha256(string: ReadableBuffer); call it with the wrong type.

typeshed contract: string is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _hashlib import openssl_sha256
try:
    openssl_sha256(_W())  # string: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_hashlib/openssl_sha384__string_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__hashlib_openssl_sha384__string_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "openssl_sha384__string_as_ReadableBuffer_wrong"
# subject = "_hashlib.openssl_sha384(string: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.openssl_sha384(string: ReadableBuffer); call it with the wrong type.

typeshed contract: string is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _hashlib import openssl_sha384
try:
    openssl_sha384(_W())  # string: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_hashlib/openssl_sha3_224__string_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__hashlib_openssl_sha3_224__string_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "openssl_sha3_224__string_as_ReadableBuffer_wrong"
# subject = "_hashlib.openssl_sha3_224(string: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.openssl_sha3_224(string: ReadableBuffer); call it with the wrong type.

typeshed contract: string is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _hashlib import openssl_sha3_224
try:
    openssl_sha3_224(_W())  # string: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_hashlib/openssl_sha3_256__string_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__hashlib_openssl_sha3_256__string_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "openssl_sha3_256__string_as_ReadableBuffer_wrong"
# subject = "_hashlib.openssl_sha3_256(string: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.openssl_sha3_256(string: ReadableBuffer); call it with the wrong type.

typeshed contract: string is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _hashlib import openssl_sha3_256
try:
    openssl_sha3_256(_W())  # string: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_hashlib/openssl_sha3_384__string_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__hashlib_openssl_sha3_384__string_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "openssl_sha3_384__string_as_ReadableBuffer_wrong"
# subject = "_hashlib.openssl_sha3_384(string: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.openssl_sha3_384(string: ReadableBuffer); call it with the wrong type.

typeshed contract: string is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _hashlib import openssl_sha3_384
try:
    openssl_sha3_384(_W())  # string: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_hashlib/openssl_sha3_512__string_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__hashlib_openssl_sha3_512__string_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "openssl_sha3_512__string_as_ReadableBuffer_wrong"
# subject = "_hashlib.openssl_sha3_512(string: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.openssl_sha3_512(string: ReadableBuffer); call it with the wrong type.

typeshed contract: string is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _hashlib import openssl_sha3_512
try:
    openssl_sha3_512(_W())  # string: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_hashlib/openssl_sha512__string_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__hashlib_openssl_sha512__string_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "openssl_sha512__string_as_ReadableBuffer_wrong"
# subject = "_hashlib.openssl_sha512(string: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.openssl_sha512(string: ReadableBuffer); call it with the wrong type.

typeshed contract: string is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _hashlib import openssl_sha512
try:
    openssl_sha512(_W())  # string: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_hashlib/openssl_shake_128__string_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__hashlib_openssl_shake_128__string_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "openssl_shake_128__string_as_ReadableBuffer_wrong"
# subject = "_hashlib.openssl_shake_128(string: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.openssl_shake_128(string: ReadableBuffer); call it with the wrong type.

typeshed contract: string is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _hashlib import openssl_shake_128
try:
    openssl_shake_128(_W())  # string: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_hashlib/openssl_shake_256__string_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__hashlib_openssl_shake_256__string_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "openssl_shake_256__string_as_ReadableBuffer_wrong"
# subject = "_hashlib.openssl_shake_256(string: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.openssl_shake_256(string: ReadableBuffer); call it with the wrong type.

typeshed contract: string is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _hashlib import openssl_shake_256
try:
    openssl_shake_256(_W())  # string: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_hashlib/pbkdf2_hmac__hash_name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__hashlib_pbkdf2_hmac__hash_name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "pbkdf2_hmac__hash_name_as_str_wrong"
# subject = "_hashlib.pbkdf2_hmac(hash_name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.pbkdf2_hmac(hash_name: str); call it with the wrong type.

typeshed contract: hash_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _hashlib import pbkdf2_hmac
try:
    pbkdf2_hmac(12345, None, None, 0)  # hash_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_hashlib/scrypt__password_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__hashlib_scrypt__password_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_hashlib"
# dimension = "type"
# case = "scrypt__password_as_ReadableBuffer_wrong"
# subject = "_hashlib.scrypt(password: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _hashlib.scrypt(password: ReadableBuffer); call it with the wrong type.

typeshed contract: password is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _hashlib import scrypt
try:
    scrypt(_W())  # password: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/hashlib/file_digest__fileobj_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_hashlib_file_digest__fileobj_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "type"
# case = "file_digest__fileobj_as_typed_wrong"
# subject = "hashlib.file_digest(fileobj: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: hashlib.file_digest(fileobj: typed); call it with the wrong type.

typeshed contract: fileobj is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from hashlib import file_digest
try:
    file_digest(_W(), None)  # fileobj: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/hashlib/new__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_hashlib_new__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "type"
# case = "new__name_as_str_wrong"
# subject = "hashlib.new(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: hashlib.new(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from hashlib import new
try:
    new(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
