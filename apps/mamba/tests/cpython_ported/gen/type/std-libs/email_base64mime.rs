use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/email_base64mime/body_encode__s_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_base64mime_body_encode__s_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_base64mime"
# dimension = "type"
# case = "body_encode__s_as_typed_wrong"
# subject = "email.base64mime.body_encode(s: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/base64mime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.base64mime.body_encode(s: typed); call it with the wrong type.

typeshed contract: s is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.base64mime import body_encode
try:
    body_encode(_W())  # s: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_base64mime/decode__string_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_base64mime_decode__string_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_base64mime"
# dimension = "type"
# case = "decode__string_as_typed_wrong"
# subject = "email.base64mime.decode(string: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/base64mime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.base64mime.decode(string: typed); call it with the wrong type.

typeshed contract: string is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.base64mime import decode
try:
    decode(_W())  # string: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_base64mime/header_encode__header_bytes_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_base64mime_header_encode__header_bytes_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_base64mime"
# dimension = "type"
# case = "header_encode__header_bytes_as_typed_wrong"
# subject = "email.base64mime.header_encode(header_bytes: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/base64mime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.base64mime.header_encode(header_bytes: typed); call it with the wrong type.

typeshed contract: header_bytes is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.base64mime import header_encode
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

/// Ported from `tests/cpython/type/std-libs/email_base64mime/header_length__bytearray_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_base64mime_header_length__bytearray_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_base64mime"
# dimension = "type"
# case = "header_length__bytearray_as_typed_wrong"
# subject = "email.base64mime.header_length(bytearray: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/base64mime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.base64mime.header_length(bytearray: typed); call it with the wrong type.

typeshed contract: bytearray is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.base64mime import header_length
try:
    header_length(_W())  # bytearray: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
