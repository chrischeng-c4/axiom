# Operational AssertionPass seed for SILENT divergences across the
# os.path attribute-surface / ipaddress instance-surface pair
# pinned by atomic 164: `os.path` (the documented `basename` /
# `dirname` / `join` / `splitext` / `normpath` / `isabs` /
# `abspath` / `exists` / `split` POSIX-style filesystem-path
# helper attribute surface) and `ipaddress` (the documented
# `ip_address` / `ip_network` constructor returning an
# `IPv4Address` / `IPv4Network` instance + `.is_loopback`
# boolean property + `.prefixlen` / `.num_addresses` network
# attributes + address-in-network containment semantics).
#
# The matching subset (posixpath full helper surface, mimetypes
# guess-type layer + module hasattr surface, ipaddress class
# identifier hasattr surface, ipaddress.version integer property,
# is_private boolean property on canonical RFC1918 input) is
# covered by `test_posixpath_mimetypes_ipaddress_value_ops`;
# this fixture pins the CPython-only contracts that mamba
# currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • hasattr(os.path, "basename") is True — documented POSIX
#     helper attribute (mamba: False — every `os.path.*` helper
#     is hasattr False, the entire surface is unreachable);
#   • hasattr(os.path, "dirname") is True (mamba: False);
#   • hasattr(os.path, "join") is True (mamba: False);
#   • hasattr(os.path, "splitext") is True (mamba: False);
#   • hasattr(os.path, "normpath") is True (mamba: False);
#   • hasattr(os.path, "isabs") is True (mamba: False);
#   • ipaddress.ip_address("192.168.1.1") returns an
#     IPv4Address instance — type(addr).__name__ ==
#     "IPv4Address" (mamba: returns an integer handle,
#     type(addr).__name__ == "int");
#   • ipaddress.ip_address("127.0.0.1").is_loopback is True —
#     documented loopback predicate (mamba: returns None,
#     predicate is a no-op shim);
#   • ipaddress.ip_address("192.168.1.1").is_loopback is False
#     — predicate inverted for non-loopback addresses
#     (mamba: returns None);
#   • ipaddress.ip_address("::1").is_loopback is True —
#     documented IPv6 loopback predicate (mamba: None);
#   • ipaddress.ip_network("192.168.1.0/24").prefixlen == 24
#     — documented prefix length attribute (mamba: returns
#     None);
#   • ipaddress.ip_network("192.168.1.0/24").num_addresses ==
#     256 — documented address-count attribute (mamba:
#     returns None);
#   • ipaddress.ip_address("192.168.1.5") in
#     ipaddress.ip_network("192.168.1.0/24") is True —
#     documented address-in-network containment contract
#     (mamba: returns False — membership is broken because
#     ip_network is an int handle, not a Network instance).
import os.path as _ospath_mod
import ipaddress as _ipaddress_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / module-level helpers / instance methods
# that mamba's bundled type stubs do not surface accurately.
os_path: Any = _ospath_mod
ipaddress: Any = _ipaddress_mod


_ledger: list[int] = []

# 1) os.path — documented helper attribute surface
assert hasattr(os_path, "basename") == True; _ledger.append(1)
assert hasattr(os_path, "dirname") == True; _ledger.append(1)
assert hasattr(os_path, "join") == True; _ledger.append(1)
assert hasattr(os_path, "splitext") == True; _ledger.append(1)
assert hasattr(os_path, "normpath") == True; _ledger.append(1)
assert hasattr(os_path, "isabs") == True; _ledger.append(1)

# 2) ipaddress.ip_address — IPv4 / IPv6 instance identity
_v4 = ipaddress.ip_address("192.168.1.1")
assert type(_v4).__name__ == "IPv4Address"; _ledger.append(1)
_v6 = ipaddress.ip_address("::1")
assert type(_v6).__name__ == "IPv6Address"; _ledger.append(1)

# 3) ipaddress — `.is_loopback` boolean predicate
assert ipaddress.ip_address("127.0.0.1").is_loopback == True; _ledger.append(1)
assert ipaddress.ip_address("192.168.1.1").is_loopback == False; _ledger.append(1)
assert ipaddress.ip_address("::1").is_loopback == True; _ledger.append(1)

# 4) ipaddress.ip_network — `.prefixlen` / `.num_addresses`
_net = ipaddress.ip_network("192.168.1.0/24")
assert _net.prefixlen == 24; _ledger.append(1)
assert _net.num_addresses == 256; _ledger.append(1)

# 5) ipaddress — address-in-network containment
assert (ipaddress.ip_address("192.168.1.5") in _net) == True; _ledger.append(1)
assert (ipaddress.ip_address("10.0.0.1") in _net) == False; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_ospath_ipaddress_instance_silent {sum(_ledger)} asserts")
