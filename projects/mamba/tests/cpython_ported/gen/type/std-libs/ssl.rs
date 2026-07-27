use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_ssl/Certificate__public_bytes__format_as_Literal_wrong.py`.
#[test]
fn test_gen_type_std_libs__ssl_Certificate__public_bytes__format_as_Literal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ssl"
# dimension = "type"
# case = "Certificate__public_bytes__format_as_Literal_wrong"
# subject = "_ssl.Certificate.public_bytes(format: Literal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ssl.Certificate.public_bytes(format: Literal); call it with the wrong type.

typeshed contract: format is Literal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _ssl import Certificate
obj = object.__new__(Certificate)
try:
    obj.public_bytes(_W())  # format: Literal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ssl/Certificate__public_bytes__format_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__ssl_Certificate__public_bytes__format_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ssl"
# dimension = "type"
# case = "Certificate__public_bytes__format_as_int_wrong"
# subject = "_ssl.Certificate.public_bytes(format: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ssl.Certificate.public_bytes(format: int); call it with the wrong type.

typeshed contract: format is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ssl import Certificate
obj = object.__new__(Certificate)
try:
    obj.public_bytes("not_an_int")  # format: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ssl/MemoryBIO__read__size_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__ssl_MemoryBIO__read__size_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ssl"
# dimension = "type"
# case = "MemoryBIO__read__size_as_int_wrong"
# subject = "_ssl.MemoryBIO.read(size: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ssl.MemoryBIO.read(size: int); call it with the wrong type.

typeshed contract: size is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ssl import MemoryBIO
obj = object.__new__(MemoryBIO)
try:
    obj.read("not_an_int")  # size: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ssl/MemoryBIO__write__b_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__ssl_MemoryBIO__write__b_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ssl"
# dimension = "type"
# case = "MemoryBIO__write__b_as_ReadableBuffer_wrong"
# subject = "_ssl.MemoryBIO.write(b: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ssl.MemoryBIO.write(b: ReadableBuffer); call it with the wrong type.

typeshed contract: b is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _ssl import MemoryBIO
obj = object.__new__(MemoryBIO)
try:
    obj.write(_W())  # b: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ssl/RAND_add__string_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__ssl_RAND_add__string_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ssl"
# dimension = "type"
# case = "RAND_add__string_as_typed_wrong"
# subject = "_ssl.RAND_add(string: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ssl.RAND_add(string: typed); call it with the wrong type.

typeshed contract: string is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _ssl import RAND_add
try:
    RAND_add(_W(), 0.0)  # string: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ssl/RAND_bytes__n_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__ssl_RAND_bytes__n_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ssl"
# dimension = "type"
# case = "RAND_bytes__n_as_int_wrong"
# subject = "_ssl.RAND_bytes(n: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ssl.RAND_bytes(n: int); call it with the wrong type.

typeshed contract: n is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ssl import RAND_bytes
try:
    RAND_bytes("not_an_int")  # n: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ssl/RAND_pseudo_bytes__n_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__ssl_RAND_pseudo_bytes__n_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ssl"
# dimension = "type"
# case = "RAND_pseudo_bytes__n_as_int_wrong"
# subject = "_ssl.RAND_pseudo_bytes(n: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ssl.RAND_pseudo_bytes(n: int); call it with the wrong type.

typeshed contract: n is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ssl import RAND_pseudo_bytes
try:
    RAND_pseudo_bytes("not_an_int")  # n: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ssl/enum_certificates__store_name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__ssl_enum_certificates__store_name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ssl"
# dimension = "type"
# case = "enum_certificates__store_name_as_str_wrong"
# subject = "_ssl.enum_certificates(store_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ssl.enum_certificates(store_name: str); call it with the wrong type.

typeshed contract: store_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ssl import enum_certificates
try:
    enum_certificates(12345)  # store_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ssl/enum_crls__store_name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__ssl_enum_crls__store_name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ssl"
# dimension = "type"
# case = "enum_crls__store_name_as_str_wrong"
# subject = "_ssl.enum_crls(store_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ssl.enum_crls(store_name: str); call it with the wrong type.

typeshed contract: store_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ssl import enum_crls
try:
    enum_crls(12345)  # store_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ssl/nid2obj__nid_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__ssl_nid2obj__nid_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ssl"
# dimension = "type"
# case = "nid2obj__nid_as_int_wrong"
# subject = "_ssl.nid2obj(nid: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ssl.nid2obj(nid: int); call it with the wrong type.

typeshed contract: nid is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ssl import nid2obj
try:
    nid2obj("not_an_int")  # nid: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_ssl/txt2obj__txt_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__ssl_txt2obj__txt_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ssl"
# dimension = "type"
# case = "txt2obj__txt_as_str_wrong"
# subject = "_ssl.txt2obj(txt: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ssl.txt2obj(txt: str); call it with the wrong type.

typeshed contract: txt is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ssl import txt2obj
try:
    txt2obj(12345)  # txt: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/DER_cert_to_PEM_cert__der_cert_bytes_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_DER_cert_to_PEM_cert__der_cert_bytes_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "DER_cert_to_PEM_cert__der_cert_bytes_as_ReadableBuffer_wrong"
# subject = "ssl.DER_cert_to_PEM_cert(der_cert_bytes: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.DER_cert_to_PEM_cert(der_cert_bytes: ReadableBuffer); call it with the wrong type.

typeshed contract: der_cert_bytes is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ssl import DER_cert_to_PEM_cert
try:
    DER_cert_to_PEM_cert(_W())  # der_cert_bytes: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/PEM_cert_to_DER_cert__pem_cert_string_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_PEM_cert_to_DER_cert__pem_cert_string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "PEM_cert_to_DER_cert__pem_cert_string_as_str_wrong"
# subject = "ssl.PEM_cert_to_DER_cert(pem_cert_string: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.PEM_cert_to_DER_cert(pem_cert_string: str); call it with the wrong type.

typeshed contract: pem_cert_string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ssl import PEM_cert_to_DER_cert
try:
    PEM_cert_to_DER_cert(12345)  # pem_cert_string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLContext__load_default_certs__purpose_as_Purpose_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLContext__load_default_certs__purpose_as_Purpose_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLContext__load_default_certs__purpose_as_Purpose_wrong"
# subject = "ssl.SSLContext.load_default_certs(purpose: Purpose)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLContext.load_default_certs(purpose: Purpose); call it with the wrong type.

typeshed contract: purpose is Purpose. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ssl import SSLContext
obj = object.__new__(SSLContext)
try:
    obj.load_default_certs(_W())  # purpose: Purpose <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLContext__load_dh_params__path_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLContext__load_dh_params__path_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLContext__load_dh_params__path_as_str_wrong"
# subject = "ssl.SSLContext.load_dh_params(path: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLContext.load_dh_params(path: str); call it with the wrong type.

typeshed contract: path is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ssl import SSLContext
obj = object.__new__(SSLContext)
try:
    obj.load_dh_params(12345)  # path: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLContext__load_verify_locations__cafile_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLContext__load_verify_locations__cafile_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLContext__load_verify_locations__cafile_as_typed_wrong"
# subject = "ssl.SSLContext.load_verify_locations(cafile: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLContext.load_verify_locations(cafile: typed); call it with the wrong type.

typeshed contract: cafile is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ssl import SSLContext
obj = object.__new__(SSLContext)
try:
    obj.load_verify_locations(_W())  # cafile: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLContext__set_alpn_protocols__alpn_protocols_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLContext__set_alpn_protocols__alpn_protocols_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLContext__set_alpn_protocols__alpn_protocols_as_Iterable_wrong"
# subject = "ssl.SSLContext.set_alpn_protocols(alpn_protocols: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLContext.set_alpn_protocols(alpn_protocols: Iterable); call it with the wrong type.

typeshed contract: alpn_protocols is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ssl import SSLContext
obj = object.__new__(SSLContext)
try:
    obj.set_alpn_protocols(_W())  # alpn_protocols: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLContext__set_ciphers__cipherlist_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLContext__set_ciphers__cipherlist_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLContext__set_ciphers__cipherlist_as_str_wrong"
# subject = "ssl.SSLContext.set_ciphers(cipherlist: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLContext.set_ciphers(cipherlist: str); call it with the wrong type.

typeshed contract: cipherlist is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ssl import SSLContext
obj = object.__new__(SSLContext)
try:
    obj.set_ciphers(12345)  # cipherlist: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLContext__set_ciphersuites__ciphersuites_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLContext__set_ciphersuites__ciphersuites_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLContext__set_ciphersuites__ciphersuites_as_str_wrong"
# subject = "ssl.SSLContext.set_ciphersuites(ciphersuites: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLContext.set_ciphersuites(ciphersuites: str); call it with the wrong type.

typeshed contract: ciphersuites is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ssl import SSLContext
obj = object.__new__(SSLContext)
try:
    obj.set_ciphersuites(12345)  # ciphersuites: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLContext__set_client_sigalgs__sigalgs_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLContext__set_client_sigalgs__sigalgs_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLContext__set_client_sigalgs__sigalgs_as_str_wrong"
# subject = "ssl.SSLContext.set_client_sigalgs(sigalgs: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLContext.set_client_sigalgs(sigalgs: str); call it with the wrong type.

typeshed contract: sigalgs is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ssl import SSLContext
obj = object.__new__(SSLContext)
try:
    obj.set_client_sigalgs(12345)  # sigalgs: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLContext__set_ecdh_curve__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLContext__set_ecdh_curve__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLContext__set_ecdh_curve__name_as_str_wrong"
# subject = "ssl.SSLContext.set_ecdh_curve(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLContext.set_ecdh_curve(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ssl import SSLContext
obj = object.__new__(SSLContext)
try:
    obj.set_ecdh_curve(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLContext__set_groups__grouplist_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLContext__set_groups__grouplist_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLContext__set_groups__grouplist_as_str_wrong"
# subject = "ssl.SSLContext.set_groups(grouplist: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLContext.set_groups(grouplist: str); call it with the wrong type.

typeshed contract: grouplist is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ssl import SSLContext
obj = object.__new__(SSLContext)
try:
    obj.set_groups(12345)  # grouplist: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLContext__set_npn_protocols__npn_protocols_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLContext__set_npn_protocols__npn_protocols_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLContext__set_npn_protocols__npn_protocols_as_Iterable_wrong"
# subject = "ssl.SSLContext.set_npn_protocols(npn_protocols: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLContext.set_npn_protocols(npn_protocols: Iterable); call it with the wrong type.

typeshed contract: npn_protocols is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ssl import SSLContext
obj = object.__new__(SSLContext)
try:
    obj.set_npn_protocols(_W())  # npn_protocols: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLContext__set_server_sigalgs__sigalgs_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLContext__set_server_sigalgs__sigalgs_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLContext__set_server_sigalgs__sigalgs_as_str_wrong"
# subject = "ssl.SSLContext.set_server_sigalgs(sigalgs: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLContext.set_server_sigalgs(sigalgs: str); call it with the wrong type.

typeshed contract: sigalgs is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ssl import SSLContext
obj = object.__new__(SSLContext)
try:
    obj.set_server_sigalgs(12345)  # sigalgs: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLContext__set_servername_callback__server_name_callback_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLContext__set_servername_callback__server_name_callback_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLContext__set_servername_callback__server_name_callback_as_typed_wrong"
# subject = "ssl.SSLContext.set_servername_callback(server_name_callback: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLContext.set_servername_callback(server_name_callback: typed); call it with the wrong type.

typeshed contract: server_name_callback is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ssl import SSLContext
obj = object.__new__(SSLContext)
try:
    obj.set_servername_callback(_W())  # server_name_callback: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLContext__wrap_bio__incoming_as_MemoryBIO_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLContext__wrap_bio__incoming_as_MemoryBIO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLContext__wrap_bio__incoming_as_MemoryBIO_wrong"
# subject = "ssl.SSLContext.wrap_bio(incoming: MemoryBIO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLContext.wrap_bio(incoming: MemoryBIO); call it with the wrong type.

typeshed contract: incoming is MemoryBIO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ssl import SSLContext
obj = object.__new__(SSLContext)
try:
    obj.wrap_bio(_W(), None)  # incoming: MemoryBIO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLContext__wrap_socket__sock_as_socket_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLContext__wrap_socket__sock_as_socket_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLContext__wrap_socket__sock_as_socket_wrong"
# subject = "ssl.SSLContext.wrap_socket(sock: socket)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLContext.wrap_socket(sock: socket); call it with the wrong type.

typeshed contract: sock is socket. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ssl import SSLContext
obj = object.__new__(SSLContext)
try:
    obj.wrap_socket(_W())  # sock: socket <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLObject__get_channel_binding__cb_type_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLObject__get_channel_binding__cb_type_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLObject__get_channel_binding__cb_type_as_str_wrong"
# subject = "ssl.SSLObject.get_channel_binding(cb_type: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLObject.get_channel_binding(cb_type: str); call it with the wrong type.

typeshed contract: cb_type is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ssl import SSLObject
obj = object.__new__(SSLObject)
try:
    obj.get_channel_binding(12345)  # cb_type: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLObject__read__len_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLObject__read__len_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLObject__read__len_as_int_wrong"
# subject = "ssl.SSLObject.read(len: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLObject.read(len: int); call it with the wrong type.

typeshed contract: len is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ssl import SSLObject
obj = object.__new__(SSLObject)
try:
    obj.read("not_an_int")  # len: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLObject__write__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLObject__write__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLObject__write__data_as_ReadableBuffer_wrong"
# subject = "ssl.SSLObject.write(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLObject.write(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ssl import SSLObject
obj = object.__new__(SSLObject)
try:
    obj.write(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLSocket__connect__addr_as__Address_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLSocket__connect__addr_as__Address_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLSocket__connect__addr_as__Address_wrong"
# subject = "ssl.SSLSocket.connect(addr: _Address)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLSocket.connect(addr: _Address); call it with the wrong type.

typeshed contract: addr is _Address. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ssl import SSLSocket
obj = object.__new__(SSLSocket)
try:
    obj.connect(_W())  # addr: _Address <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLSocket__connect_ex__addr_as__Address_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLSocket__connect_ex__addr_as__Address_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLSocket__connect_ex__addr_as__Address_wrong"
# subject = "ssl.SSLSocket.connect_ex(addr: _Address)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLSocket.connect_ex(addr: _Address); call it with the wrong type.

typeshed contract: addr is _Address. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ssl import SSLSocket
obj = object.__new__(SSLSocket)
try:
    obj.connect_ex(_W())  # addr: _Address <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLSocket__get_channel_binding__cb_type_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLSocket__get_channel_binding__cb_type_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLSocket__get_channel_binding__cb_type_as_str_wrong"
# subject = "ssl.SSLSocket.get_channel_binding(cb_type: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLSocket.get_channel_binding(cb_type: str); call it with the wrong type.

typeshed contract: cb_type is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ssl import SSLSocket
obj = object.__new__(SSLSocket)
try:
    obj.get_channel_binding(12345)  # cb_type: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLSocket__read__len_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLSocket__read__len_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLSocket__read__len_as_int_wrong"
# subject = "ssl.SSLSocket.read(len: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLSocket.read(len: int); call it with the wrong type.

typeshed contract: len is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ssl import SSLSocket
obj = object.__new__(SSLSocket)
try:
    obj.read("not_an_int")  # len: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLSocket__recv__buflen_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLSocket__recv__buflen_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLSocket__recv__buflen_as_int_wrong"
# subject = "ssl.SSLSocket.recv(buflen: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLSocket.recv(buflen: int); call it with the wrong type.

typeshed contract: buflen is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ssl import SSLSocket
obj = object.__new__(SSLSocket)
try:
    obj.recv("not_an_int")  # buflen: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLSocket__recv_into__buffer_as_WriteableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLSocket__recv_into__buffer_as_WriteableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLSocket__recv_into__buffer_as_WriteableBuffer_wrong"
# subject = "ssl.SSLSocket.recv_into(buffer: WriteableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLSocket.recv_into(buffer: WriteableBuffer); call it with the wrong type.

typeshed contract: buffer is WriteableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ssl import SSLSocket
obj = object.__new__(SSLSocket)
try:
    obj.recv_into(_W())  # buffer: WriteableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLSocket__recvfrom__buflen_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLSocket__recvfrom__buflen_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLSocket__recvfrom__buflen_as_int_wrong"
# subject = "ssl.SSLSocket.recvfrom(buflen: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLSocket.recvfrom(buflen: int); call it with the wrong type.

typeshed contract: buflen is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ssl import SSLSocket
obj = object.__new__(SSLSocket)
try:
    obj.recvfrom("not_an_int")  # buflen: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLSocket__recvfrom_into__buffer_as_WriteableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLSocket__recvfrom_into__buffer_as_WriteableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLSocket__recvfrom_into__buffer_as_WriteableBuffer_wrong"
# subject = "ssl.SSLSocket.recvfrom_into(buffer: WriteableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLSocket.recvfrom_into(buffer: WriteableBuffer); call it with the wrong type.

typeshed contract: buffer is WriteableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ssl import SSLSocket
obj = object.__new__(SSLSocket)
try:
    obj.recvfrom_into(_W())  # buffer: WriteableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLSocket__send__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLSocket__send__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLSocket__send__data_as_ReadableBuffer_wrong"
# subject = "ssl.SSLSocket.send(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLSocket.send(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ssl import SSLSocket
obj = object.__new__(SSLSocket)
try:
    obj.send(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLSocket__sendall__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLSocket__sendall__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLSocket__sendall__data_as_ReadableBuffer_wrong"
# subject = "ssl.SSLSocket.sendall(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLSocket.sendall(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ssl import SSLSocket
obj = object.__new__(SSLSocket)
try:
    obj.sendall(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLSocket__shutdown__how_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLSocket__shutdown__how_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLSocket__shutdown__how_as_int_wrong"
# subject = "ssl.SSLSocket.shutdown(how: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLSocket.shutdown(how: int); call it with the wrong type.

typeshed contract: how is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ssl import SSLSocket
obj = object.__new__(SSLSocket)
try:
    obj.shutdown("not_an_int")  # how: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/SSLSocket__write__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_SSLSocket__write__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLSocket__write__data_as_ReadableBuffer_wrong"
# subject = "ssl.SSLSocket.write(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLSocket.write(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ssl import SSLSocket
obj = object.__new__(SSLSocket)
try:
    obj.write(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/cert_time_to_seconds__cert_time_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_cert_time_to_seconds__cert_time_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "cert_time_to_seconds__cert_time_as_str_wrong"
# subject = "ssl.cert_time_to_seconds(cert_time: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.cert_time_to_seconds(cert_time: str); call it with the wrong type.

typeshed contract: cert_time is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ssl import cert_time_to_seconds
try:
    cert_time_to_seconds(12345)  # cert_time: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/create_default_context__purpose_as_Purpose_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_create_default_context__purpose_as_Purpose_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "create_default_context__purpose_as_Purpose_wrong"
# subject = "ssl.create_default_context(purpose: Purpose)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.create_default_context(purpose: Purpose); call it with the wrong type.

typeshed contract: purpose is Purpose. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ssl import create_default_context
try:
    create_default_context(_W())  # purpose: Purpose <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/get_protocol_name__protocol_code_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_get_protocol_name__protocol_code_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "get_protocol_name__protocol_code_as_int_wrong"
# subject = "ssl.get_protocol_name(protocol_code: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.get_protocol_name(protocol_code: int); call it with the wrong type.

typeshed contract: protocol_code is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ssl import get_protocol_name
try:
    get_protocol_name("not_an_int")  # protocol_code: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/match_hostname__cert_as__PeerCertRetDictType_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_match_hostname__cert_as__PeerCertRetDictType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "match_hostname__cert_as__PeerCertRetDictType_wrong"
# subject = "ssl.match_hostname(cert: _PeerCertRetDictType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.match_hostname(cert: _PeerCertRetDictType); call it with the wrong type.

typeshed contract: cert is _PeerCertRetDictType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ssl import match_hostname
try:
    match_hostname(_W(), "")  # cert: _PeerCertRetDictType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ssl/wrap_socket__sock_as_socket_wrong.py`.
#[test]
fn test_gen_type_std_libs_ssl_wrap_socket__sock_as_socket_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "wrap_socket__sock_as_socket_wrong"
# subject = "ssl.wrap_socket(sock: socket)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.wrap_socket(sock: socket); call it with the wrong type.

typeshed contract: sock is socket. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ssl import wrap_socket
try:
    wrap_socket(_W())  # sock: socket <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
