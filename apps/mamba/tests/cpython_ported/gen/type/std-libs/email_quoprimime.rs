use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/email_quoprimime/body_check__octet_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_quoprimime_body_check__octet_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_quoprimime"
# dimension = "type"
# case = "body_check__octet_as_int_wrong"
# subject = "email.quoprimime.body_check(octet: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/quoprimime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.quoprimime.body_check(octet: int); call it with the wrong type.

typeshed contract: octet is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.quoprimime import body_check
try:
    body_check("not_an_int")  # octet: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_quoprimime/body_encode__body_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_quoprimime_body_encode__body_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_quoprimime"
# dimension = "type"
# case = "body_encode__body_as_str_wrong"
# subject = "email.quoprimime.body_encode(body: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/quoprimime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.quoprimime.body_encode(body: str); call it with the wrong type.

typeshed contract: body is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.quoprimime import body_encode
try:
    body_encode(12345)  # body: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_quoprimime/body_length__bytearray_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_quoprimime_body_length__bytearray_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_quoprimime"
# dimension = "type"
# case = "body_length__bytearray_as_Iterable_wrong"
# subject = "email.quoprimime.body_length(bytearray: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/quoprimime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.quoprimime.body_length(bytearray: Iterable); call it with the wrong type.

typeshed contract: bytearray is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.quoprimime import body_length
try:
    body_length(_W())  # bytearray: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_quoprimime/decode__encoded_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_quoprimime_decode__encoded_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_quoprimime"
# dimension = "type"
# case = "decode__encoded_as_str_wrong"
# subject = "email.quoprimime.decode(encoded: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/quoprimime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.quoprimime.decode(encoded: str); call it with the wrong type.

typeshed contract: encoded is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.quoprimime import decode
try:
    decode(12345)  # encoded: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_quoprimime/header_check__octet_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_quoprimime_header_check__octet_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_quoprimime"
# dimension = "type"
# case = "header_check__octet_as_int_wrong"
# subject = "email.quoprimime.header_check(octet: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/quoprimime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.quoprimime.header_check(octet: int); call it with the wrong type.

typeshed contract: octet is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.quoprimime import header_check
try:
    header_check("not_an_int")  # octet: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_quoprimime/header_decode__s_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_quoprimime_header_decode__s_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_quoprimime"
# dimension = "type"
# case = "header_decode__s_as_str_wrong"
# subject = "email.quoprimime.header_decode(s: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/quoprimime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.quoprimime.header_decode(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.quoprimime import header_decode
try:
    header_decode(12345)  # s: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_quoprimime/header_encode__header_bytes_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_quoprimime_header_encode__header_bytes_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_quoprimime"
# dimension = "type"
# case = "header_encode__header_bytes_as_typed_wrong"
# subject = "email.quoprimime.header_encode(header_bytes: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/quoprimime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.quoprimime.header_encode(header_bytes: typed); call it with the wrong type.

typeshed contract: header_bytes is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.quoprimime import header_encode
try:
    header_encode(_W())  # header_bytes: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_quoprimime/header_length__bytearray_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_quoprimime_header_length__bytearray_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_quoprimime"
# dimension = "type"
# case = "header_length__bytearray_as_Iterable_wrong"
# subject = "email.quoprimime.header_length(bytearray: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/quoprimime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.quoprimime.header_length(bytearray: Iterable); call it with the wrong type.

typeshed contract: bytearray is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.quoprimime import header_length
try:
    header_length(_W())  # bytearray: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_quoprimime/quote__c_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_quoprimime_quote__c_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_quoprimime"
# dimension = "type"
# case = "quote__c_as_typed_wrong"
# subject = "email.quoprimime.quote(c: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/quoprimime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.quoprimime.quote(c: typed); call it with the wrong type.

typeshed contract: c is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.quoprimime import quote
try:
    quote(_W())  # c: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_quoprimime/unquote__s_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_quoprimime_unquote__s_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_quoprimime"
# dimension = "type"
# case = "unquote__s_as_typed_wrong"
# subject = "email.quoprimime.unquote(s: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/quoprimime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.quoprimime.unquote(s: typed); call it with the wrong type.

typeshed contract: s is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.quoprimime import unquote
try:
    unquote(_W())  # s: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
