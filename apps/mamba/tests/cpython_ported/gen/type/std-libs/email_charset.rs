use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/email_charset/Charset__body_encode__string_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_charset_Charset__body_encode__string_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_charset"
# dimension = "type"
# case = "Charset__body_encode__string_as_typed_wrong"
# subject = "email.charset.Charset.body_encode(string: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/charset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.charset.Charset.body_encode(string: typed); call it with the wrong type.

typeshed contract: string is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.charset import Charset
obj = object.__new__(Charset)
try:
    obj.body_encode(_W())  # string: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_charset/Charset__header_encode__string_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_charset_Charset__header_encode__string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_charset"
# dimension = "type"
# case = "Charset__header_encode__string_as_str_wrong"
# subject = "email.charset.Charset.header_encode(string: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/charset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.charset.Charset.header_encode(string: str); call it with the wrong type.

typeshed contract: string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.charset import Charset
obj = object.__new__(Charset)
try:
    obj.header_encode(12345)  # string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_charset/Charset__header_encode_lines__string_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_charset_Charset__header_encode_lines__string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_charset"
# dimension = "type"
# case = "Charset__header_encode_lines__string_as_str_wrong"
# subject = "email.charset.Charset.header_encode_lines(string: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/charset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.charset.Charset.header_encode_lines(string: str); call it with the wrong type.

typeshed contract: string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.charset import Charset
obj = object.__new__(Charset)
try:
    obj.header_encode_lines(12345, None)  # string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_charset/Charset__init__input_charset_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_charset_Charset__init__input_charset_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_charset"
# dimension = "type"
# case = "Charset__init__input_charset_as_str_wrong"
# subject = "email.charset.Charset.__init__(input_charset: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/charset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.charset.Charset.__init__(input_charset: str); call it with the wrong type.

typeshed contract: input_charset is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.charset import Charset
try:
    Charset(12345)  # input_charset: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_charset/add_alias__alias_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_charset_add_alias__alias_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_charset"
# dimension = "type"
# case = "add_alias__alias_as_str_wrong"
# subject = "email.charset.add_alias(alias: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/charset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.charset.add_alias(alias: str); call it with the wrong type.

typeshed contract: alias is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.charset import add_alias
try:
    add_alias(12345, "")  # alias: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_charset/add_charset__charset_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_charset_add_charset__charset_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_charset"
# dimension = "type"
# case = "add_charset__charset_as_str_wrong"
# subject = "email.charset.add_charset(charset: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/charset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.charset.add_charset(charset: str); call it with the wrong type.

typeshed contract: charset is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.charset import add_charset
try:
    add_charset(12345)  # charset: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_charset/add_codec__charset_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_charset_add_codec__charset_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_charset"
# dimension = "type"
# case = "add_codec__charset_as_str_wrong"
# subject = "email.charset.add_codec(charset: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/charset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.charset.add_codec(charset: str); call it with the wrong type.

typeshed contract: charset is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.charset import add_codec
try:
    add_codec(12345, "")  # charset: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
