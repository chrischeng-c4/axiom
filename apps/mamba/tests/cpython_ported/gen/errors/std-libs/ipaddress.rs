use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/ipaddress/address_value_error_is_value_error.py`.
#[test]
fn test_gen_errors_std_libs_ipaddress_address_value_error_is_value_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "errors"
# case = "address_value_error_is_value_error"
# subject = "ipaddress.AddressValueError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.AddressValueError: AddressValueError is a subclass of ValueError so a plain `except ValueError` catches it"""
import ipaddress

assert issubclass(ipaddress.AddressValueError, ValueError), "AddressValueError <: ValueError"
_caught = False
try:
    ipaddress.IPv4Address("256.0.0.1")
except ValueError as e:
    _caught = type(e).__name__ == "AddressValueError"
assert _caught, "plain except ValueError catches AddressValueError"
print("address_value_error_is_value_error OK")
"###);
    assert_output(&out, r###"address_value_error_is_value_error OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ipaddress/bad_ip_string_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_ipaddress_bad_ip_string_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "errors"
# case = "bad_ip_string_raises_valueerror"
# subject = "ipaddress.ip_address"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.ip_address: bad_ip_string_raises_valueerror (errors)."""
import ipaddress

_raised = False
try:
    ipaddress.ip_address("not.an.ip")
except ValueError:
    _raised = True
assert _raised, "bad_ip_string_raises_valueerror: expected ValueError"
print("bad_ip_string_raises_valueerror OK")
"###);
    assert_output(&out, r###"bad_ip_string_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ipaddress/invalid_ipv4_octet_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_ipaddress_invalid_ipv4_octet_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "errors"
# case = "invalid_ipv4_octet_raises_valueerror"
# subject = "ipaddress.ip_address"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.ip_address: invalid_ipv4_octet_raises_valueerror (errors)."""
import ipaddress

_raised = False
try:
    ipaddress.ip_address("999.0.0.1")
except ValueError:
    _raised = True
assert _raised, "invalid_ipv4_octet_raises_valueerror: expected ValueError"
print("invalid_ipv4_octet_raises_valueerror OK")
"###);
    assert_output(&out, r###"invalid_ipv4_octet_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ipaddress/ipv4address_octet_over_255_raises.py`.
#[test]
fn test_gen_errors_std_libs_ipaddress_ipv4address_octet_over_255_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "errors"
# case = "ipv4address_octet_over_255_raises"
# subject = "ipaddress.IPv4Address"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.IPv4Address: ipv4address_octet_over_255_raises (errors)."""
import ipaddress

_raised = False
try:
    ipaddress.IPv4Address("256.0.0.1")
except ipaddress.AddressValueError:
    _raised = True
assert _raised, "ipv4address_octet_over_255_raises: expected ipaddress.AddressValueError"
print("ipv4address_octet_over_255_raises OK")
"###);
    assert_output(&out, r###"ipv4address_octet_over_255_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ipaddress/ipv6address_bad_colon_raises.py`.
#[test]
fn test_gen_errors_std_libs_ipaddress_ipv6address_bad_colon_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "errors"
# case = "ipv6address_bad_colon_raises"
# subject = "ipaddress.IPv6Address"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.IPv6Address: ipv6address_bad_colon_raises (errors)."""
import ipaddress

_raised = False
try:
    ipaddress.IPv6Address(":::1")
except ipaddress.AddressValueError:
    _raised = True
assert _raised, "ipv6address_bad_colon_raises: expected ipaddress.AddressValueError"
print("ipv6address_bad_colon_raises OK")
"###);
    assert_output(&out, r###"ipv6address_bad_colon_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ipaddress/netmask_value_error_is_value_error.py`.
#[test]
fn test_gen_errors_std_libs_ipaddress_netmask_value_error_is_value_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "errors"
# case = "netmask_value_error_is_value_error"
# subject = "ipaddress.NetmaskValueError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.NetmaskValueError: NetmaskValueError is a subclass of ValueError so a plain `except ValueError` catches it"""
import ipaddress

assert issubclass(ipaddress.NetmaskValueError, ValueError), "NetmaskValueError <: ValueError"
_caught = False
try:
    ipaddress.IPv4Network("10.0.0.0/40")
except ValueError as e:
    _caught = type(e).__name__ == "NetmaskValueError"
assert _caught, "plain except ValueError catches NetmaskValueError"
print("netmask_value_error_is_value_error OK")
"###);
    assert_output(&out, r###"netmask_value_error_is_value_error OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ipaddress/prefix_over_32_raises_netmaskvalueerror.py`.
#[test]
fn test_gen_errors_std_libs_ipaddress_prefix_over_32_raises_netmaskvalueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "errors"
# case = "prefix_over_32_raises_netmaskvalueerror"
# subject = "ipaddress.IPv4Network"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.IPv4Network: prefix_over_32_raises_netmaskvalueerror (errors)."""
import ipaddress

_raised = False
try:
    ipaddress.IPv4Network("10.0.0.0/40")
except ipaddress.NetmaskValueError:
    _raised = True
assert _raised, "prefix_over_32_raises_netmaskvalueerror: expected ipaddress.NetmaskValueError"
print("prefix_over_32_raises_netmaskvalueerror OK")
"###);
    assert_output(&out, r###"prefix_over_32_raises_netmaskvalueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ipaddress/strict_host_bits_set_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_ipaddress_strict_host_bits_set_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "errors"
# case = "strict_host_bits_set_raises_valueerror"
# subject = "ipaddress.IPv4Network"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.IPv4Network: strict_host_bits_set_raises_valueerror (errors)."""
import ipaddress

_raised = False
try:
    ipaddress.IPv4Network("10.0.0.1/24", strict=True)
except ValueError:
    _raised = True
assert _raised, "strict_host_bits_set_raises_valueerror: expected ValueError"
print("strict_host_bits_set_raises_valueerror OK")
"###);
    assert_output(&out, r###"strict_host_bits_set_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ipaddress/subnet_of_cross_version_raises_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_ipaddress_subnet_of_cross_version_raises_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "errors"
# case = "subnet_of_cross_version_raises_typeerror"
# subject = "ipaddress.IPv4Network"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.IPv4Network: subnet_of_cross_version_raises_typeerror (errors)."""
import ipaddress

_raised = False
try:
    ipaddress.IPv4Network("10.0.0.0/30").subnet_of(ipaddress.IPv6Network("::1/128"))
except TypeError:
    _raised = True
assert _raised, "subnet_of_cross_version_raises_typeerror: expected TypeError"
print("subnet_of_cross_version_raises_typeerror OK")
"###);
    assert_output(&out, r###"subnet_of_cross_version_raises_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ipaddress/supernet_of_cross_version_raises_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_ipaddress_supernet_of_cross_version_raises_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "errors"
# case = "supernet_of_cross_version_raises_typeerror"
# subject = "ipaddress.IPv6Network"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.IPv6Network: supernet_of_cross_version_raises_typeerror (errors)."""
import ipaddress

_raised = False
try:
    ipaddress.IPv6Network("::1/128").supernet_of(ipaddress.IPv4Network("10.0.0.0/30"))
except TypeError:
    _raised = True
assert _raised, "supernet_of_cross_version_raises_typeerror: expected TypeError"
print("supernet_of_cross_version_raises_typeerror OK")
"###);
    assert_output(&out, r###"supernet_of_cross_version_raises_typeerror OK
"###);
}
