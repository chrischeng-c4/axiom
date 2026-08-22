use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/AddressHeader__init__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_AddressHeader__init__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "AddressHeader__init__name_as_str_wrong"
# subject = "email.headerregistry.AddressHeader.init(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.AddressHeader.init(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import AddressHeader
obj = object.__new__(AddressHeader)
try:
    obj.init(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/AddressHeader__parse__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_AddressHeader__parse__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "AddressHeader__parse__value_as_str_wrong"
# subject = "email.headerregistry.AddressHeader.parse(value: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.AddressHeader.parse(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import AddressHeader
try:
    AddressHeader.parse(12345, None)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/AddressHeader__value_parser__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_AddressHeader__value_parser__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "AddressHeader__value_parser__value_as_str_wrong"
# subject = "email.headerregistry.AddressHeader.value_parser(value: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.AddressHeader.value_parser(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import AddressHeader
try:
    AddressHeader.value_parser(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/Address__init__display_name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_Address__init__display_name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "Address__init__display_name_as_str_wrong"
# subject = "email.headerregistry.Address.__init__(display_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.Address.__init__(display_name: str); call it with the wrong type.

typeshed contract: display_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import Address
try:
    Address(12345)  # display_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/BaseHeader____new____name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_BaseHeader____new____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "BaseHeader____new____name_as_str_wrong"
# subject = "email.headerregistry.BaseHeader.__new__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.BaseHeader.__new__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import BaseHeader
obj = object.__new__(BaseHeader)
try:
    obj.__new__(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/BaseHeader__init__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_BaseHeader__init__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "BaseHeader__init__name_as_str_wrong"
# subject = "email.headerregistry.BaseHeader.init(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.BaseHeader.init(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import BaseHeader
obj = object.__new__(BaseHeader)
try:
    obj.init(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/ContentDispositionHeader__value_parser__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_ContentDispositionHeader__value_parser__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "ContentDispositionHeader__value_parser__value_as_str_wrong"
# subject = "email.headerregistry.ContentDispositionHeader.value_parser(value: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.ContentDispositionHeader.value_parser(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import ContentDispositionHeader
try:
    ContentDispositionHeader.value_parser(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/ContentTransferEncodingHeader__init__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_ContentTransferEncodingHeader__init__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "ContentTransferEncodingHeader__init__name_as_str_wrong"
# subject = "email.headerregistry.ContentTransferEncodingHeader.init(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.ContentTransferEncodingHeader.init(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import ContentTransferEncodingHeader
obj = object.__new__(ContentTransferEncodingHeader)
try:
    obj.init(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/ContentTransferEncodingHeader__parse__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_ContentTransferEncodingHeader__parse__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "ContentTransferEncodingHeader__parse__value_as_str_wrong"
# subject = "email.headerregistry.ContentTransferEncodingHeader.parse(value: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.ContentTransferEncodingHeader.parse(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import ContentTransferEncodingHeader
try:
    ContentTransferEncodingHeader.parse(12345, None)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/ContentTransferEncodingHeader__value_parser__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_ContentTransferEncodingHeader__value_parser__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "ContentTransferEncodingHeader__value_parser__value_as_str_wrong"
# subject = "email.headerregistry.ContentTransferEncodingHeader.value_parser(value: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.ContentTransferEncodingHeader.value_parser(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import ContentTransferEncodingHeader
try:
    ContentTransferEncodingHeader.value_parser(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/ContentTypeHeader__value_parser__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_ContentTypeHeader__value_parser__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "ContentTypeHeader__value_parser__value_as_str_wrong"
# subject = "email.headerregistry.ContentTypeHeader.value_parser(value: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.ContentTypeHeader.value_parser(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import ContentTypeHeader
try:
    ContentTypeHeader.value_parser(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/DateHeader__init__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_DateHeader__init__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "DateHeader__init__name_as_str_wrong"
# subject = "email.headerregistry.DateHeader.init(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.DateHeader.init(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import DateHeader
obj = object.__new__(DateHeader)
try:
    obj.init(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/DateHeader__parse__value_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_DateHeader__parse__value_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "DateHeader__parse__value_as_typed_wrong"
# subject = "email.headerregistry.DateHeader.parse(value: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.DateHeader.parse(value: typed); call it with the wrong type.

typeshed contract: value is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.headerregistry import DateHeader
try:
    DateHeader.parse(_W(), None)  # value: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/DateHeader__value_parser__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_DateHeader__value_parser__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "DateHeader__value_parser__value_as_str_wrong"
# subject = "email.headerregistry.DateHeader.value_parser(value: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.DateHeader.value_parser(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import DateHeader
try:
    DateHeader.value_parser(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/Group__init__display_name_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_Group__init__display_name_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "Group__init__display_name_as_typed_wrong"
# subject = "email.headerregistry.Group.__init__(display_name: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.Group.__init__(display_name: typed); call it with the wrong type.

typeshed contract: display_name is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.headerregistry import Group
try:
    Group(_W())  # display_name: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/HeaderRegistry____call____name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_HeaderRegistry____call____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "HeaderRegistry____call____name_as_str_wrong"
# subject = "email.headerregistry.HeaderRegistry.__call__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.HeaderRegistry.__call__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import HeaderRegistry
obj = object.__new__(HeaderRegistry)
try:
    obj.__call__(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/HeaderRegistry____getitem____name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_HeaderRegistry____getitem____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "HeaderRegistry____getitem____name_as_str_wrong"
# subject = "email.headerregistry.HeaderRegistry.__getitem__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.HeaderRegistry.__getitem__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import HeaderRegistry
obj = object.__new__(HeaderRegistry)
try:
    obj.__getitem__(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/HeaderRegistry__init__base_class_as_type_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_HeaderRegistry__init__base_class_as_type_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "HeaderRegistry__init__base_class_as_type_wrong"
# subject = "email.headerregistry.HeaderRegistry.__init__(base_class: type)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.HeaderRegistry.__init__(base_class: type); call it with the wrong type.

typeshed contract: base_class is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.headerregistry import HeaderRegistry
try:
    HeaderRegistry(_W())  # base_class: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/HeaderRegistry__map_to_type__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_HeaderRegistry__map_to_type__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "HeaderRegistry__map_to_type__name_as_str_wrong"
# subject = "email.headerregistry.HeaderRegistry.map_to_type(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.HeaderRegistry.map_to_type(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import HeaderRegistry
obj = object.__new__(HeaderRegistry)
try:
    obj.map_to_type(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/MIMEVersionHeader__init__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_MIMEVersionHeader__init__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "MIMEVersionHeader__init__name_as_str_wrong"
# subject = "email.headerregistry.MIMEVersionHeader.init(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.MIMEVersionHeader.init(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import MIMEVersionHeader
obj = object.__new__(MIMEVersionHeader)
try:
    obj.init(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/MIMEVersionHeader__parse__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_MIMEVersionHeader__parse__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "MIMEVersionHeader__parse__value_as_str_wrong"
# subject = "email.headerregistry.MIMEVersionHeader.parse(value: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.MIMEVersionHeader.parse(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import MIMEVersionHeader
try:
    MIMEVersionHeader.parse(12345, None)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/MIMEVersionHeader__value_parser__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_MIMEVersionHeader__value_parser__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "MIMEVersionHeader__value_parser__value_as_str_wrong"
# subject = "email.headerregistry.MIMEVersionHeader.value_parser(value: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.MIMEVersionHeader.value_parser(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import MIMEVersionHeader
try:
    MIMEVersionHeader.value_parser(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/MessageIDHeader__parse__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_MessageIDHeader__parse__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "MessageIDHeader__parse__value_as_str_wrong"
# subject = "email.headerregistry.MessageIDHeader.parse(value: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.MessageIDHeader.parse(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import MessageIDHeader
try:
    MessageIDHeader.parse(12345, None)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/MessageIDHeader__value_parser__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_MessageIDHeader__value_parser__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "MessageIDHeader__value_parser__value_as_str_wrong"
# subject = "email.headerregistry.MessageIDHeader.value_parser(value: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.MessageIDHeader.value_parser(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import MessageIDHeader
try:
    MessageIDHeader.value_parser(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/ParameterizedMIMEHeader__init__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_ParameterizedMIMEHeader__init__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "ParameterizedMIMEHeader__init__name_as_str_wrong"
# subject = "email.headerregistry.ParameterizedMIMEHeader.init(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.ParameterizedMIMEHeader.init(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import ParameterizedMIMEHeader
obj = object.__new__(ParameterizedMIMEHeader)
try:
    obj.init(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/ParameterizedMIMEHeader__parse__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_ParameterizedMIMEHeader__parse__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "ParameterizedMIMEHeader__parse__value_as_str_wrong"
# subject = "email.headerregistry.ParameterizedMIMEHeader.parse(value: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.ParameterizedMIMEHeader.parse(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import ParameterizedMIMEHeader
try:
    ParameterizedMIMEHeader.parse(12345, None)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/ReferencesHeader__parse__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_ReferencesHeader__parse__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "ReferencesHeader__parse__value_as_str_wrong"
# subject = "email.headerregistry.ReferencesHeader.parse(value: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.ReferencesHeader.parse(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import ReferencesHeader
try:
    ReferencesHeader.parse(12345, None)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/ReferencesHeader__value_parser__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_ReferencesHeader__value_parser__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "ReferencesHeader__value_parser__value_as_str_wrong"
# subject = "email.headerregistry.ReferencesHeader.value_parser(value: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.ReferencesHeader.value_parser(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import ReferencesHeader
try:
    ReferencesHeader.value_parser(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/UnstructuredHeader__parse__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_UnstructuredHeader__parse__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "UnstructuredHeader__parse__value_as_str_wrong"
# subject = "email.headerregistry.UnstructuredHeader.parse(value: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.UnstructuredHeader.parse(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import UnstructuredHeader
try:
    UnstructuredHeader.parse(12345, None)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email_headerregistry/UnstructuredHeader__value_parser__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email_headerregistry_UnstructuredHeader__value_parser__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_headerregistry"
# dimension = "type"
# case = "UnstructuredHeader__value_parser__value_as_str_wrong"
# subject = "email.headerregistry.UnstructuredHeader.value_parser(value: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/headerregistry.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.headerregistry.UnstructuredHeader.value_parser(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email.headerregistry import UnstructuredHeader
try:
    UnstructuredHeader.value_parser(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
