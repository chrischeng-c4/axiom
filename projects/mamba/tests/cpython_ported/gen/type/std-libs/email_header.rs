use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/email_header/Header__append__s_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_header_Header__append__s_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_header"
# dimension = "type"
# case = "Header__append__s_as_typed_wrong"
# subject = "email.header.Header.append(s: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/header.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.header.Header.append(s: typed); call it with the wrong type.

typeshed contract: s is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.header import Header
obj = object.__new__(Header)
try:
    obj.append(_W())  # s: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_header/Header__encode__splitchars_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_header_Header__encode__splitchars_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_header"
# dimension = "type"
# case = "Header__encode__splitchars_as_str_wrong"
# subject = "email.header.Header.encode(splitchars: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/header.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.header.Header.encode(splitchars: str); call it with the wrong type.

typeshed contract: splitchars is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.header import Header
obj = object.__new__(Header)
try:
    obj.encode(12345)  # splitchars: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_header/Header__init__s_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_header_Header__init__s_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_header"
# dimension = "type"
# case = "Header__init__s_as_typed_wrong"
# subject = "email.header.Header.__init__(s: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/header.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.header.Header.__init__(s: typed); call it with the wrong type.

typeshed contract: s is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.header import Header
try:
    Header(_W())  # s: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_header/decode_header__header_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_header_decode_header__header_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_header"
# dimension = "type"
# case = "decode_header__header_as_typed_wrong"
# subject = "email.header.decode_header(header: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/header.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.header.decode_header(header: typed); call it with the wrong type.

typeshed contract: header is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.header import decode_header
try:
    decode_header(_W())  # header: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_header/make_header__decoded_seq_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_header_make_header__decoded_seq_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_header"
# dimension = "type"
# case = "make_header__decoded_seq_as_Iterable_wrong"
# subject = "email.header.make_header(decoded_seq: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/header.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.header.make_header(decoded_seq: Iterable); call it with the wrong type.

typeshed contract: decoded_seq is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.header import make_header
try:
    make_header(_W())  # decoded_seq: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
