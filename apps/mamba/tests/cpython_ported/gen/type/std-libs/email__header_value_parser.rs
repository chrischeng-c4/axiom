use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/MsgID__fold__policy_as_Policy_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_MsgID__fold__policy_as_Policy_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "MsgID__fold__policy_as_Policy_wrong"
# subject = "email._header_value_parser.MsgID.fold(policy: Policy)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.MsgID.fold(policy: Policy); call it with the wrong type.

typeshed contract: policy is Policy. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email._header_value_parser import MsgID
obj = object.__new__(MsgID)
try:
    obj.fold(_W())  # policy: Policy <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/Terminal____new____value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_Terminal____new____value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "Terminal____new____value_as_str_wrong"
# subject = "email._header_value_parser.Terminal.__new__(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.Terminal.__new__(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import Terminal
obj = object.__new__(Terminal)
try:
    obj.__new__(12345, "")  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/TokenList__pprint__indent_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_TokenList__pprint__indent_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "TokenList__pprint__indent_as_str_wrong"
# subject = "email._header_value_parser.TokenList.pprint(indent: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.TokenList.pprint(indent: str); call it with the wrong type.

typeshed contract: indent is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import TokenList
obj = object.__new__(TokenList)
try:
    obj.pprint(12345)  # indent: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/TokenList__ppstr__indent_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_TokenList__ppstr__indent_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "TokenList__ppstr__indent_as_str_wrong"
# subject = "email._header_value_parser.TokenList.ppstr(indent: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.TokenList.ppstr(indent: str); call it with the wrong type.

typeshed contract: indent is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import TokenList
obj = object.__new__(TokenList)
try:
    obj.ppstr(12345)  # indent: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_addr_spec__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_addr_spec__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_addr_spec__value_as_str_wrong"
# subject = "email._header_value_parser.get_addr_spec(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_addr_spec(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_addr_spec
try:
    get_addr_spec(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_address__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_address__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_address__value_as_str_wrong"
# subject = "email._header_value_parser.get_address(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_address(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_address
try:
    get_address(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_address_list__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_address_list__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_address_list__value_as_str_wrong"
# subject = "email._header_value_parser.get_address_list(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_address_list(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_address_list
try:
    get_address_list(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_angle_addr__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_angle_addr__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_angle_addr__value_as_str_wrong"
# subject = "email._header_value_parser.get_angle_addr(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_angle_addr(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_angle_addr
try:
    get_angle_addr(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_atext__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_atext__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_atext__value_as_str_wrong"
# subject = "email._header_value_parser.get_atext(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_atext(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_atext
try:
    get_atext(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_atom__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_atom__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_atom__value_as_str_wrong"
# subject = "email._header_value_parser.get_atom(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_atom(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_atom
try:
    get_atom(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_attribute__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_attribute__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_attribute__value_as_str_wrong"
# subject = "email._header_value_parser.get_attribute(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_attribute(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_attribute
try:
    get_attribute(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_attrtext__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_attrtext__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_attrtext__value_as_str_wrong"
# subject = "email._header_value_parser.get_attrtext(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_attrtext(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_attrtext
try:
    get_attrtext(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_bare_quoted_string__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_bare_quoted_string__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_bare_quoted_string__value_as_str_wrong"
# subject = "email._header_value_parser.get_bare_quoted_string(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_bare_quoted_string(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_bare_quoted_string
try:
    get_bare_quoted_string(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_cfws__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_cfws__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_cfws__value_as_str_wrong"
# subject = "email._header_value_parser.get_cfws(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_cfws(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_cfws
try:
    get_cfws(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_comment__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_comment__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_comment__value_as_str_wrong"
# subject = "email._header_value_parser.get_comment(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_comment(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_comment
try:
    get_comment(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_display_name__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_display_name__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_display_name__value_as_str_wrong"
# subject = "email._header_value_parser.get_display_name(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_display_name(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_display_name
try:
    get_display_name(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_domain__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_domain__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_domain__value_as_str_wrong"
# subject = "email._header_value_parser.get_domain(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_domain(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_domain
try:
    get_domain(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_domain_literal__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_domain_literal__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_domain_literal__value_as_str_wrong"
# subject = "email._header_value_parser.get_domain_literal(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_domain_literal(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_domain_literal
try:
    get_domain_literal(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_dot_atom__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_dot_atom__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_dot_atom__value_as_str_wrong"
# subject = "email._header_value_parser.get_dot_atom(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_dot_atom(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_dot_atom
try:
    get_dot_atom(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_dot_atom_text__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_dot_atom_text__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_dot_atom_text__value_as_str_wrong"
# subject = "email._header_value_parser.get_dot_atom_text(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_dot_atom_text(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_dot_atom_text
try:
    get_dot_atom_text(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_dtext__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_dtext__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_dtext__value_as_str_wrong"
# subject = "email._header_value_parser.get_dtext(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_dtext(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_dtext
try:
    get_dtext(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_encoded_word__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_encoded_word__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_encoded_word__value_as_str_wrong"
# subject = "email._header_value_parser.get_encoded_word(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_encoded_word(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_encoded_word
try:
    get_encoded_word(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_extended_attribute__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_extended_attribute__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_extended_attribute__value_as_str_wrong"
# subject = "email._header_value_parser.get_extended_attribute(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_extended_attribute(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_extended_attribute
try:
    get_extended_attribute(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_extended_attrtext__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_extended_attrtext__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_extended_attrtext__value_as_str_wrong"
# subject = "email._header_value_parser.get_extended_attrtext(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_extended_attrtext(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_extended_attrtext
try:
    get_extended_attrtext(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_fws__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_fws__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_fws__value_as_str_wrong"
# subject = "email._header_value_parser.get_fws(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_fws(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_fws
try:
    get_fws(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_group__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_group__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_group__value_as_str_wrong"
# subject = "email._header_value_parser.get_group(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_group(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_group
try:
    get_group(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_group_list__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_group_list__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_group_list__value_as_str_wrong"
# subject = "email._header_value_parser.get_group_list(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_group_list(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_group_list
try:
    get_group_list(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_invalid_mailbox__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_invalid_mailbox__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_invalid_mailbox__value_as_str_wrong"
# subject = "email._header_value_parser.get_invalid_mailbox(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_invalid_mailbox(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_invalid_mailbox
try:
    get_invalid_mailbox(12345, "")  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_invalid_parameter__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_invalid_parameter__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_invalid_parameter__value_as_str_wrong"
# subject = "email._header_value_parser.get_invalid_parameter(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_invalid_parameter(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_invalid_parameter
try:
    get_invalid_parameter(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_local_part__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_local_part__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_local_part__value_as_str_wrong"
# subject = "email._header_value_parser.get_local_part(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_local_part(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_local_part
try:
    get_local_part(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_mailbox__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_mailbox__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_mailbox__value_as_str_wrong"
# subject = "email._header_value_parser.get_mailbox(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_mailbox(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_mailbox
try:
    get_mailbox(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_mailbox_list__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_mailbox_list__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_mailbox_list__value_as_str_wrong"
# subject = "email._header_value_parser.get_mailbox_list(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_mailbox_list(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_mailbox_list
try:
    get_mailbox_list(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_msg_id__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_msg_id__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_msg_id__value_as_str_wrong"
# subject = "email._header_value_parser.get_msg_id(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_msg_id(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_msg_id
try:
    get_msg_id(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_name_addr__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_name_addr__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_name_addr__value_as_str_wrong"
# subject = "email._header_value_parser.get_name_addr(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_name_addr(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_name_addr
try:
    get_name_addr(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_no_fold_literal__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_no_fold_literal__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_no_fold_literal__value_as_str_wrong"
# subject = "email._header_value_parser.get_no_fold_literal(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_no_fold_literal(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_no_fold_literal
try:
    get_no_fold_literal(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_obs_local_part__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_obs_local_part__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_obs_local_part__value_as_str_wrong"
# subject = "email._header_value_parser.get_obs_local_part(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_obs_local_part(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_obs_local_part
try:
    get_obs_local_part(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_obs_route__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_obs_route__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_obs_route__value_as_str_wrong"
# subject = "email._header_value_parser.get_obs_route(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_obs_route(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_obs_route
try:
    get_obs_route(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_parameter__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_parameter__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_parameter__value_as_str_wrong"
# subject = "email._header_value_parser.get_parameter(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_parameter(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_parameter
try:
    get_parameter(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_phrase__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_phrase__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_phrase__value_as_str_wrong"
# subject = "email._header_value_parser.get_phrase(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_phrase(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_phrase
try:
    get_phrase(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_qcontent__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_qcontent__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_qcontent__value_as_str_wrong"
# subject = "email._header_value_parser.get_qcontent(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_qcontent(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_qcontent
try:
    get_qcontent(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_qp_ctext__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_qp_ctext__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_qp_ctext__value_as_str_wrong"
# subject = "email._header_value_parser.get_qp_ctext(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_qp_ctext(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_qp_ctext
try:
    get_qp_ctext(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_quoted_string__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_quoted_string__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_quoted_string__value_as_str_wrong"
# subject = "email._header_value_parser.get_quoted_string(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_quoted_string(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_quoted_string
try:
    get_quoted_string(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_section__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_section__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_section__value_as_str_wrong"
# subject = "email._header_value_parser.get_section(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_section(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_section
try:
    get_section(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_token__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_token__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_token__value_as_str_wrong"
# subject = "email._header_value_parser.get_token(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_token(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_token
try:
    get_token(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_ttext__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_ttext__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_ttext__value_as_str_wrong"
# subject = "email._header_value_parser.get_ttext(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_ttext(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_ttext
try:
    get_ttext(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_unstructured__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_unstructured__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_unstructured__value_as_str_wrong"
# subject = "email._header_value_parser.get_unstructured(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_unstructured(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_unstructured
try:
    get_unstructured(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_value__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_value__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_value__value_as_str_wrong"
# subject = "email._header_value_parser.get_value(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_value(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_value
try:
    get_value(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/get_word__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_get_word__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "get_word__value_as_str_wrong"
# subject = "email._header_value_parser.get_word(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.get_word(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import get_word
try:
    get_word(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/parse_content_disposition_header__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_parse_content_disposition_header__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "parse_content_disposition_header__value_as_str_wrong"
# subject = "email._header_value_parser.parse_content_disposition_header(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.parse_content_disposition_header(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import parse_content_disposition_header
try:
    parse_content_disposition_header(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/parse_content_transfer_encoding_header__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_parse_content_transfer_encoding_header__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "parse_content_transfer_encoding_header__value_as_str_wrong"
# subject = "email._header_value_parser.parse_content_transfer_encoding_header(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.parse_content_transfer_encoding_header(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import parse_content_transfer_encoding_header
try:
    parse_content_transfer_encoding_header(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/parse_content_type_header__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_parse_content_type_header__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "parse_content_type_header__value_as_str_wrong"
# subject = "email._header_value_parser.parse_content_type_header(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.parse_content_type_header(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import parse_content_type_header
try:
    parse_content_type_header(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/parse_message_id__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_parse_message_id__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "parse_message_id__value_as_str_wrong"
# subject = "email._header_value_parser.parse_message_id(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.parse_message_id(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import parse_message_id
try:
    parse_message_id(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/parse_message_ids__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_parse_message_ids__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "parse_message_ids__value_as_str_wrong"
# subject = "email._header_value_parser.parse_message_ids(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.parse_message_ids(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import parse_message_ids
try:
    parse_message_ids(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/parse_mime_parameters__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_parse_mime_parameters__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "parse_mime_parameters__value_as_str_wrong"
# subject = "email._header_value_parser.parse_mime_parameters(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.parse_mime_parameters(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import parse_mime_parameters
try:
    parse_mime_parameters(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/email__header_value_parser/parse_mime_version__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_email__header_value_parser_parse_mime_version__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__header_value_parser"
# dimension = "type"
# case = "parse_mime_version__value_as_str_wrong"
# subject = "email._header_value_parser.parse_mime_version(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_header_value_parser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._header_value_parser.parse_mime_version(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._header_value_parser import parse_mime_version
try:
    parse_mime_version(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
