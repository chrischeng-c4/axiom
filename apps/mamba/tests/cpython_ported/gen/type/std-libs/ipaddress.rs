use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/ipaddress/IPv4Network__init__strict_as_bool_wrong.py`.
#[test]
fn test_gen_type_std_libs_ipaddress_IPv4Network__init__strict_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "type"
# case = "IPv4Network__init__strict_as_bool_wrong"
# subject = "ipaddress.IPv4Network.__init__(strict: bool)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ipaddress.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ipaddress.IPv4Network.__init__(strict: bool); call it with the wrong type.

typeshed contract: strict is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ipaddress import IPv4Network
try:
    IPv4Network(None, "not_a_bool")  # strict: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ipaddress/IPv6Network__init__strict_as_bool_wrong.py`.
#[test]
fn test_gen_type_std_libs_ipaddress_IPv6Network__init__strict_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "type"
# case = "IPv6Network__init__strict_as_bool_wrong"
# subject = "ipaddress.IPv6Network.__init__(strict: bool)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ipaddress.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ipaddress.IPv6Network.__init__(strict: bool); call it with the wrong type.

typeshed contract: strict is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ipaddress import IPv6Network
try:
    IPv6Network(None, "not_a_bool")  # strict: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ipaddress/collapse_addresses__addresses_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_ipaddress_collapse_addresses__addresses_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "type"
# case = "collapse_addresses__addresses_as_Iterable_wrong"
# subject = "ipaddress.collapse_addresses(addresses: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ipaddress.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ipaddress.collapse_addresses(addresses: Iterable); call it with the wrong type.

typeshed contract: addresses is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ipaddress import collapse_addresses
try:
    collapse_addresses(_W())  # addresses: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ipaddress/get_mixed_type_key__obj_as_IPv4Network_wrong.py`.
#[test]
fn test_gen_type_std_libs_ipaddress_get_mixed_type_key__obj_as_IPv4Network_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "type"
# case = "get_mixed_type_key__obj_as_IPv4Network_wrong"
# subject = "ipaddress.get_mixed_type_key(obj: IPv4Network)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ipaddress.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ipaddress.get_mixed_type_key(obj: IPv4Network); call it with the wrong type.

typeshed contract: obj is IPv4Network. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ipaddress import get_mixed_type_key
try:
    get_mixed_type_key(_W())  # obj: IPv4Network <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ipaddress/get_mixed_type_key__obj_as_IPv6Network_wrong.py`.
#[test]
fn test_gen_type_std_libs_ipaddress_get_mixed_type_key__obj_as_IPv6Network_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "type"
# case = "get_mixed_type_key__obj_as_IPv6Network_wrong"
# subject = "ipaddress.get_mixed_type_key(obj: IPv6Network)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ipaddress.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ipaddress.get_mixed_type_key(obj: IPv6Network); call it with the wrong type.

typeshed contract: obj is IPv6Network. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ipaddress import get_mixed_type_key
try:
    get_mixed_type_key(_W())  # obj: IPv6Network <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ipaddress/get_mixed_type_key__obj_as__A_wrong.py`.
#[test]
fn test_gen_type_std_libs_ipaddress_get_mixed_type_key__obj_as__A_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "type"
# case = "get_mixed_type_key__obj_as__A_wrong"
# subject = "ipaddress.get_mixed_type_key(obj: _A)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ipaddress.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ipaddress.get_mixed_type_key(obj: _A); call it with the wrong type.

typeshed contract: obj is _A. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ipaddress import get_mixed_type_key
try:
    get_mixed_type_key(_W())  # obj: _A <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ipaddress/ip_address__address_as__RawIPAddress_wrong.py`.
#[test]
fn test_gen_type_std_libs_ipaddress_ip_address__address_as__RawIPAddress_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "type"
# case = "ip_address__address_as__RawIPAddress_wrong"
# subject = "ipaddress.ip_address(address: _RawIPAddress)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ipaddress.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ipaddress.ip_address(address: _RawIPAddress); call it with the wrong type.

typeshed contract: address is _RawIPAddress. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ipaddress import ip_address
try:
    ip_address(_W())  # address: _RawIPAddress <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ipaddress/ip_interface__address_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_ipaddress_ip_interface__address_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "type"
# case = "ip_interface__address_as_typed_wrong"
# subject = "ipaddress.ip_interface(address: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ipaddress.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ipaddress.ip_interface(address: typed); call it with the wrong type.

typeshed contract: address is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ipaddress import ip_interface
try:
    ip_interface(_W())  # address: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ipaddress/ip_network__address_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_ipaddress_ip_network__address_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "type"
# case = "ip_network__address_as_typed_wrong"
# subject = "ipaddress.ip_network(address: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ipaddress.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ipaddress.ip_network(address: typed); call it with the wrong type.

typeshed contract: address is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ipaddress import ip_network
try:
    ip_network(_W())  # address: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ipaddress/summarize_address_range__first_as_IPv4Address_wrong.py`.
#[test]
fn test_gen_type_std_libs_ipaddress_summarize_address_range__first_as_IPv4Address_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "type"
# case = "summarize_address_range__first_as_IPv4Address_wrong"
# subject = "ipaddress.summarize_address_range(first: IPv4Address)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ipaddress.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ipaddress.summarize_address_range(first: IPv4Address); call it with the wrong type.

typeshed contract: first is IPv4Address. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ipaddress import summarize_address_range
try:
    summarize_address_range(_W(), None)  # first: IPv4Address <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ipaddress/summarize_address_range__first_as_IPv6Address_wrong.py`.
#[test]
fn test_gen_type_std_libs_ipaddress_summarize_address_range__first_as_IPv6Address_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "type"
# case = "summarize_address_range__first_as_IPv6Address_wrong"
# subject = "ipaddress.summarize_address_range(first: IPv6Address)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ipaddress.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ipaddress.summarize_address_range(first: IPv6Address); call it with the wrong type.

typeshed contract: first is IPv6Address. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ipaddress import summarize_address_range
try:
    summarize_address_range(_W(), None)  # first: IPv6Address <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ipaddress/summarize_address_range__first_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_ipaddress_summarize_address_range__first_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "type"
# case = "summarize_address_range__first_as_typed_wrong"
# subject = "ipaddress.summarize_address_range(first: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ipaddress.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ipaddress.summarize_address_range(first: typed); call it with the wrong type.

typeshed contract: first is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ipaddress import summarize_address_range
try:
    summarize_address_range(_W(), None)  # first: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ipaddress/v4_int_to_packed__address_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_ipaddress_v4_int_to_packed__address_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "type"
# case = "v4_int_to_packed__address_as_int_wrong"
# subject = "ipaddress.v4_int_to_packed(address: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ipaddress.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ipaddress.v4_int_to_packed(address: int); call it with the wrong type.

typeshed contract: address is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ipaddress import v4_int_to_packed
try:
    v4_int_to_packed("not_an_int")  # address: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ipaddress/v6_int_to_packed__address_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_ipaddress_v6_int_to_packed__address_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "type"
# case = "v6_int_to_packed__address_as_int_wrong"
# subject = "ipaddress.v6_int_to_packed(address: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ipaddress.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ipaddress.v6_int_to_packed(address: int); call it with the wrong type.

typeshed contract: address is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ipaddress import v6_int_to_packed
try:
    v6_int_to_packed("not_an_int")  # address: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
