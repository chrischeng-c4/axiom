use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/address_length_constants.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_address_length_constants() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "address_length_constants"
# subject = "ipaddress.IPV4LENGTH"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPV4LENGTH: the IPV4LENGTH / IPV6LENGTH bit-width constants are 32 and 128"""
import ipaddress

assert ipaddress.IPV4LENGTH == 32, ipaddress.IPV4LENGTH
assert ipaddress.IPV6LENGTH == 128, ipaddress.IPV6LENGTH
print("address_length_constants OK")
"###);
    assert_output(&out, r###"address_length_constants OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/collapse_addresses_merges_adjacent.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_collapse_addresses_merges_adjacent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "collapse_addresses_merges_adjacent"
# subject = "ipaddress.collapse_addresses"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.collapse_addresses: collapse_addresses merges two adjacent /25s into one /24"""
import ipaddress

addrs = [
    ipaddress.IPv4Network("192.168.1.0/25"),
    ipaddress.IPv4Network("192.168.1.128/25"),
]
collapsed = list(ipaddress.collapse_addresses(addrs))
assert len(collapsed) == 1, len(collapsed)
assert str(collapsed[0]) == "192.168.1.0/24", str(collapsed[0])
print("collapse_addresses_merges_adjacent OK")
"###);
    assert_output(&out, r###"collapse_addresses_merges_adjacent OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/global_address_classification.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_global_address_classification() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "global_address_classification"
# subject = "ipaddress.ip_address"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.ip_address: a public address (8.8.8.8) is_global and not is_private"""
import ipaddress

a = ipaddress.ip_address("8.8.8.8")
assert a.is_global, "is_global"
assert not a.is_private, "not is_private"
print("global_address_classification OK")
"###);
    assert_output(&out, r###"global_address_classification OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/ip_address_dispatches_ipv4.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_ip_address_dispatches_ipv4() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ip_address_dispatches_ipv4"
# subject = "ipaddress.ip_address"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.ip_address: ip_address of a dotted-quad string returns an IPv4Address whose str, packed, and int forms match the input"""
import ipaddress

a = ipaddress.ip_address("192.168.1.1")
assert isinstance(a, ipaddress.IPv4Address), type(a)
assert str(a) == "192.168.1.1", str(a)
assert a.packed == b"\xc0\xa8\x01\x01", a.packed
assert int(a) == 0xC0A80101, int(a)
print("ip_address_dispatches_ipv4 OK")
"###);
    assert_output(&out, r###"ip_address_dispatches_ipv4 OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/ip_address_dispatches_ipv6.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_ip_address_dispatches_ipv6() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ip_address_dispatches_ipv6"
# subject = "ipaddress.ip_address"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.ip_address: ip_address of a colon-hex string returns an IPv6Address whose str is the compressed form"""
import ipaddress

a = ipaddress.ip_address("fe80::1")
assert isinstance(a, ipaddress.IPv6Address), type(a)
assert str(ipaddress.ip_address("::1")) == "::1", "compressed loopback"
print("ip_address_dispatches_ipv6 OK")
"###);
    assert_output(&out, r###"ip_address_dispatches_ipv6 OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/ipv4_format_spec_variants.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_ipv4_format_spec_variants() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipv4_format_spec_variants"
# subject = "ipaddress.IPv4Address"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.IPv4Address: format() on an IPv4Address honors b/x/X, grouping '_', the alternate '#' prefix, 'n' (== 'b'), and 's'/'' textual forms"""
import ipaddress

v4 = ipaddress.IPv4Address("1.2.3.42")
cases = {
    "b": "00000001000000100000001100101010",
    "x": "0102032a",
    "X": "0102032A",
    "_x": "0102_032a",
    "#x": "0x0102032a",
    "#X": "0X0102032A",
    "#_X": "0X0102_032A",
    "s": "1.2.3.42",
    "": "1.2.3.42",
}
for spec, want in cases.items():
    got = format(v4, spec)
    assert got == want, (spec, got, want)
assert format(v4, "n") == format(v4, "b"), "v4 n == b"
print("ipv4_format_spec_variants OK")
"###);
    assert_output(&out, r###"ipv4_format_spec_variants OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/ipv4_interface_netmask_spellings.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_ipv4_interface_netmask_spellings() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipv4_interface_netmask_spellings"
# subject = "ipaddress.IPv4Interface"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.IPv4Interface: IPv4Interface netmask spellings (tuple int/str, dotted mask, slash mask) all collapse to the prefix form"""
import ipaddress

assert str(ipaddress.IPv4Interface(("192.0.2.0", 24))) == "192.0.2.0/24", "tuple int"
assert str(ipaddress.IPv4Interface(("192.0.2.0", "24"))) == "192.0.2.0/24", "tuple str"
assert str(ipaddress.IPv4Interface(("192.0.2.0", "255.255.255.0"))) == "192.0.2.0/24", "tuple mask"
assert str(ipaddress.IPv4Interface("192.0.2.0/255.255.255.0")) == "192.0.2.0/24", "slash mask"
print("ipv4_interface_netmask_spellings OK")
"###);
    assert_output(&out, r###"ipv4_interface_netmask_spellings OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/ipv4_interface_no_mask_host_route.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_ipv4_interface_no_mask_host_route() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipv4_interface_no_mask_host_route"
# subject = "ipaddress.IPv4Interface"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.IPv4Interface: an IPv4Interface with no mask (from str/int/bytes) is a /32 with all-ones netmask and zero hostmask"""
import ipaddress

for addr in ("1.2.3.4", 16909060, b"\x01\x02\x03\x04"):
    iface = ipaddress.IPv4Interface(addr)
    assert str(iface) == "1.2.3.4/32", str(iface)
    assert str(iface.netmask) == "255.255.255.255", "netmask"
    assert str(iface.hostmask) == "0.0.0.0", "hostmask"
print("ipv4_interface_no_mask_host_route OK")
"###);
    assert_output(&out, r###"ipv4_interface_no_mask_host_route OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/ipv4_interface_prefix_netmask_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_ipv4_interface_prefix_netmask_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipv4_interface_prefix_netmask_roundtrip"
# subject = "ipaddress.IPv4Interface"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.IPv4Interface: for every IPv4 prefix length 0..32 the prefix and netmask spellings of an interface round-trip to the same text"""
import ipaddress

for i in range(0, 33):
    base = "0.0.0.0/%d" % i
    iface = ipaddress.IPv4Interface(base)
    assert str(iface) == base, ("prefix", i, str(iface))
    rt = ipaddress.IPv4Interface("0.0.0.0/%s" % iface.netmask)
    assert str(rt) == base, ("netmask", i, str(rt))
print("ipv4_interface_prefix_netmask_roundtrip OK")
"###);
    assert_output(&out, r###"ipv4_interface_prefix_netmask_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/ipv4_mapped_ipv6_extraction.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_ipv4_mapped_ipv6_extraction() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipv4_mapped_ipv6_extraction"
# subject = "ipaddress.IPv6Address"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv6Address: an IPv4-mapped IPv6 address exposes the embedded IPv4Address via .ipv4_mapped"""
import ipaddress

a = ipaddress.IPv6Address("::ffff:192.168.1.1")
assert a.ipv4_mapped == ipaddress.IPv4Address("192.168.1.1"), a.ipv4_mapped
print("ipv4_mapped_ipv6_extraction OK")
"###);
    assert_output(&out, r###"ipv4_mapped_ipv6_extraction OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/ipv4_ordering_and_int_arithmetic.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_ipv4_ordering_and_int_arithmetic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipv4_ordering_and_int_arithmetic"
# subject = "ipaddress.IPv4Address"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv4Address: adjacent IPv4 addresses order correctly and differ by 1 under int() conversion and addition"""
import ipaddress

a1 = ipaddress.IPv4Address("192.168.1.1")
a2 = ipaddress.IPv4Address("192.168.1.2")
assert a1 < a2, "ordering"
assert int(a2) - int(a1) == 1, int(a2) - int(a1)
assert int(a1) + 1 == int(a2), "int arithmetic"
print("ipv4_ordering_and_int_arithmetic OK")
"###);
    assert_output(&out, r###"ipv4_ordering_and_int_arithmetic OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/ipv4_version_and_packed_length.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_ipv4_version_and_packed_length() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipv4_version_and_packed_length"
# subject = "ipaddress.ip_address"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.ip_address: an IPv4 address reports version 4, a 4-byte packed form, and its compressed text equals the input"""
import ipaddress

a = ipaddress.ip_address("192.168.1.5")
assert a.version == 4, a.version
assert len(a.packed) == 4, len(a.packed)
assert a.compressed == "192.168.1.5", a.compressed
print("ipv4_version_and_packed_length OK")
"###);
    assert_output(&out, r###"ipv4_version_and_packed_length OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/ipv6_exploded_full_form.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_ipv6_exploded_full_form() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipv6_exploded_full_form"
# subject = "ipaddress.IPv6Address"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv6Address: IPv6Address('::1').exploded is the fully zero-padded eight-group form"""
import ipaddress

a = ipaddress.IPv6Address("::1")
assert a.exploded == "0000:0000:0000:0000:0000:0000:0000:0001", a.exploded
print("ipv6_exploded_full_form OK")
"###);
    assert_output(&out, r###"ipv6_exploded_full_form OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/ipv6_format_spec_variants.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_ipv6_format_spec_variants() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipv6_format_spec_variants"
# subject = "ipaddress.IPv6Address"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.IPv6Address: format() on an IPv6Address packs to 32 nibbles for hex specs, groups with '_', and uses the compressed text for 's'/''; binary form is 128 bits wide"""
import ipaddress

v6 = ipaddress.IPv6Address("::1.2.3.42")
assert format(v6, "x") == "0000000000000000000000000102032a", "v6 x"
assert format(v6, "X") == "0000000000000000000000000102032A", "v6 X"
assert format(v6, "_x") == "0000_0000_0000_0000_0000_0000_0102_032a", "v6 _x"
assert format(v6, "#x") == "0x0000000000000000000000000102032a", "v6 #x"
assert format(v6, "s") == "::102:32a", "v6 s"
assert format(v6, "") == "::102:32a", "v6 default"
assert len(format(v6, "b")) == 128, "v6 binary width"
print("ipv6_format_spec_variants OK")
"###);
    assert_output(&out, r###"ipv6_format_spec_variants OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/ipv6_interface_no_mask_and_scope.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_ipv6_interface_no_mask_and_scope() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipv6_interface_no_mask_and_scope"
# subject = "ipaddress.IPv6Interface"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.IPv6Interface: an IPv6Interface defaults to /128 with all-ones netmask, and scope identifiers survive in the interface text"""
import ipaddress

for addr in ("::1", 1, b"\x00" * 15 + b"\x01"):
    iface = ipaddress.IPv6Interface(addr)
    assert str(iface) == "::1/128", str(iface)
    assert str(iface.netmask) == "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff", "netmask"
    assert str(iface.hostmask) == "::", "hostmask"
scoped = ipaddress.IPv6Interface("::1%scope")
assert str(scoped) == "::1%scope/128", str(scoped)
print("ipv6_interface_no_mask_and_scope OK")
"###);
    assert_output(&out, r###"ipv6_interface_no_mask_and_scope OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/ipv6_version_and_packed_length.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_ipv6_version_and_packed_length() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipv6_version_and_packed_length"
# subject = "ipaddress.ip_address"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.ip_address: an IPv6 address reports version 6, a 16-byte packed form, and compresses ::1 correctly"""
import ipaddress

a = ipaddress.ip_address("::1")
assert a.version == 6, a.version
assert len(a.packed) == 16, len(a.packed)
assert a.compressed == "::1", a.compressed
print("ipv6_version_and_packed_length OK")
"###);
    assert_output(&out, r###"ipv6_version_and_packed_length OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/loopback_address_classification.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_loopback_address_classification() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "loopback_address_classification"
# subject = "ipaddress.ip_address"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.ip_address: 127.0.0.1 is_loopback while 8.8.8.8 is not"""
import ipaddress

assert ipaddress.ip_address("127.0.0.1").is_loopback, "loopback"
assert not ipaddress.ip_address("8.8.8.8").is_loopback, "not loopback"
print("loopback_address_classification OK")
"###);
    assert_output(&out, r###"loopback_address_classification OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/network_address_netmask_count.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_network_address_netmask_count() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "network_address_netmask_count"
# subject = "ipaddress.IPv4Network"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv4Network: a /24 reports prefixlen 24, network_address, dotted netmask, and 256 num_addresses"""
import ipaddress

net = ipaddress.IPv4Network("192.168.1.0/24")
assert net.prefixlen == 24, net.prefixlen
assert str(net.network_address) == "192.168.1.0", str(net.network_address)
assert str(net.netmask) == "255.255.255.0", str(net.netmask)
assert net.num_addresses == 256, net.num_addresses
print("network_address_netmask_count OK")
"###);
    assert_output(&out, r###"network_address_netmask_count OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/network_broadcast_address.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_network_broadcast_address() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "network_broadcast_address"
# subject = "ipaddress.IPv4Network"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv4Network: the broadcast_address of 192.168.1.0/24 is 192.168.1.255"""
import ipaddress

net = ipaddress.IPv4Network("192.168.1.0/24")
assert str(net.broadcast_address) == "192.168.1.255", str(net.broadcast_address)
print("network_broadcast_address OK")
"###);
    assert_output(&out, r###"network_broadcast_address OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/network_from_str_int_bytes_host_route.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_network_from_str_int_bytes_host_route() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "network_from_str_int_bytes_host_route"
# subject = "ipaddress.IPv4Network"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv4Network: IPv4Network/IPv6Network accept str, int, and bytes and default to a host route (/32, /128) with no mask"""
import ipaddress

for addr in ("1.2.3.4", 16909060, b"\x01\x02\x03\x04"):
    n4 = ipaddress.IPv4Network(addr)
    assert str(n4) == "1.2.3.4/32", (type(addr).__name__, str(n4))
for addr in ("::1", 1, b"\x00" * 15 + b"\x01"):
    n6 = ipaddress.IPv6Network(addr)
    assert str(n6) == "::1/128", (type(addr).__name__, str(n6))
print("network_from_str_int_bytes_host_route OK")
"###);
    assert_output(&out, r###"network_from_str_int_bytes_host_route OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/network_hosts_excludes_endpoints.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_network_hosts_excludes_endpoints() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "network_hosts_excludes_endpoints"
# subject = "ipaddress.IPv4Network"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv4Network: hosts() of a /30 yields the 2 usable addresses, excluding the network and broadcast addresses"""
import ipaddress

net = ipaddress.IPv4Network("192.168.1.0/30")
hosts = list(net.hosts())
assert len(hosts) == 2, len(hosts)
assert str(hosts[0]) == "192.168.1.1", str(hosts[0])
assert str(hosts[1]) == "192.168.1.2", str(hosts[1])
print("network_hosts_excludes_endpoints OK")
"###);
    assert_output(&out, r###"network_hosts_excludes_endpoints OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/network_membership_contains.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_network_membership_contains() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "network_membership_contains"
# subject = "ipaddress.IPv4Network"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv4Network: an address inside a /24 tests `in` True and an address outside tests `in` False"""
import ipaddress

net = ipaddress.IPv4Network("192.168.1.0/24")
assert ipaddress.ip_address("192.168.1.100") in net, "addr in network"
assert ipaddress.ip_address("10.0.0.1") not in net, "addr not in network"
print("network_membership_contains OK")
"###);
    assert_output(&out, r###"network_membership_contains OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/network_netmask_spellings.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_network_netmask_spellings() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "network_netmask_spellings"
# subject = "ipaddress.IPv4Network"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv4Network: a netmask given as prefix int, mask string, dotted netmask, or dotted hostmask all collapse to the prefix form"""
import ipaddress

assert str(ipaddress.IPv4Network(("192.0.2.0", 24))) == "192.0.2.0/24", "net tuple int"
assert str(ipaddress.IPv4Network(("192.0.2.0", "255.255.255.0"))) == "192.0.2.0/24", "net tuple mask"
assert str(ipaddress.IPv4Network("192.0.2.0/255.255.255.0")) == "192.0.2.0/24", "net slash mask"
assert str(ipaddress.IPv4Network("0.0.0.0/0.255.255.255")) == "0.0.0.0/8", "net hostmask"
print("network_netmask_spellings OK")
"###);
    assert_output(&out, r###"network_netmask_spellings OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/network_overlaps_predicate.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_network_overlaps_predicate() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "network_overlaps_predicate"
# subject = "ipaddress.IPv4Network"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv4Network: overlaps() is True for a network and its enclosed half and False for a disjoint network"""
import ipaddress

n1 = ipaddress.IPv4Network("192.168.1.0/24")
n2 = ipaddress.IPv4Network("192.168.1.128/25")
n3 = ipaddress.IPv4Network("10.0.0.0/8")
assert n1.overlaps(n2), "overlapping networks"
assert not n1.overlaps(n3), "non-overlapping networks"
print("network_overlaps_predicate OK")
"###);
    assert_output(&out, r###"network_overlaps_predicate OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/network_subnets_split.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_network_subnets_split() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "network_subnets_split"
# subject = "ipaddress.IPv4Network"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv4Network: subnets() splits a /23 into two /24 children"""
import ipaddress

parent = ipaddress.IPv4Network("192.168.0.0/23")
subnets = list(parent.subnets())
assert len(subnets) == 2, len(subnets)
assert subnets[0].prefixlen == 24, subnets[0].prefixlen
assert str(subnets[0]) == "192.168.0.0/24", str(subnets[0])
assert str(subnets[1]) == "192.168.1.0/24", str(subnets[1])
print("network_subnets_split OK")
"###);
    assert_output(&out, r###"network_subnets_split OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/private_address_classification.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_private_address_classification() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "private_address_classification"
# subject = "ipaddress.ip_address"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.ip_address: an RFC1918 address (192.168.1.1) is_private and not is_global"""
import ipaddress

a = ipaddress.ip_address("192.168.1.1")
assert a.is_private, "is_private"
assert not a.is_global, "not is_global"
print("private_address_classification OK")
"###);
    assert_output(&out, r###"private_address_classification OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/summarize_address_range_one_block.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_summarize_address_range_one_block() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "summarize_address_range_one_block"
# subject = "ipaddress.summarize_address_range"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.summarize_address_range: summarize_address_range over 192.168.1.0..255 yields the single block 192.168.1.0/24"""
import ipaddress

summary = list(ipaddress.summarize_address_range(
    ipaddress.IPv4Address("192.168.1.0"),
    ipaddress.IPv4Address("192.168.1.255"),
))
assert len(summary) == 1, len(summary)
assert str(summary[0]) == "192.168.1.0/24", str(summary[0])
print("summarize_address_range_one_block OK")
"###);
    assert_output(&out, r###"summarize_address_range_one_block OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ipaddress/v4_int_to_packed_big_endian.py`.
#[test]
fn test_gen_behavior_std_libs_ipaddress_v4_int_to_packed_big_endian() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "v4_int_to_packed_big_endian"
# subject = "ipaddress.v4_int_to_packed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.v4_int_to_packed: v4_int_to_packed(0xC0A80101) is the 4-byte big-endian b'\\xc0\\xa8\\x01\\x01'"""
import ipaddress

pkt = ipaddress.v4_int_to_packed(0xC0A80101)
assert pkt == b"\xc0\xa8\x01\x01", pkt
assert len(pkt) == 4, len(pkt)
assert (pkt[0], pkt[1], pkt[2], pkt[3]) == (192, 168, 1, 1), tuple(pkt)
print("v4_int_to_packed_big_endian OK")
"###);
    assert_output(&out, r###"v4_int_to_packed_big_endian OK
"###);
}
