# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_posixpath_mimetypes_ipaddress_value_ops"
# subject = "cpython321.test_posixpath_mimetypes_ipaddress_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_posixpath_mimetypes_ipaddress_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_posixpath_mimetypes_ipaddress_value_ops: execute CPython 3.12 seed test_posixpath_mimetypes_ipaddress_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of three
# bootstrap stdlib modules used by every POSIX-path / MIME-type
# / IP-address path: `posixpath` (the documented `basename` /
# `dirname` / `join` / `splitext` / `normpath` / `split` /
# `isabs` POSIX-style filesystem-path helpers), `mimetypes`
# (the documented `guess_type` / `guess_extension` / `init` /
# `add_type` MIME-lookup helpers — exercised against canonical
# `.html` / `.txt` / `.jpg` / `.png` extension surface), and
# `ipaddress` (the documented `ip_address` / `ip_network` /
# `IPv4Address` / `IPv6Address` / `IPv4Network` / `IPv6Network`
# class-identifier attribute surface + the documented
# `is_private` / `version` instance-attribute surface).
#
# The matching subset between mamba and CPython is the posixpath
# full helper layer, the mimetypes guess-type layer + module
# hasattr surface, the ipaddress class-identifier hasattr layer,
# and the documented `.version` integer property + `.is_private`
# boolean property on canonical RFC1918 / IPv6-loopback inputs.
#
# Surface in this fixture:
#   • posixpath — basename / dirname / join / splitext /
#     normpath / split / isabs + module hasattr surface;
#   • mimetypes — guess_type for html / txt / jpg / png /
#     unknown extension + module hasattr surface;
#   • ipaddress — `ip_address` / `ip_network` / `IPv4Address` /
#     `IPv6Address` / `IPv4Network` / `IPv6Network` class
#     identifier hasattr surface + IPv4 / IPv6 `.version`
#     integer property + IPv4 RFC1918 `.is_private` property.
#
# Behavioral edges that DIVERGE on mamba (entire `os.path`
# attribute surface absent — `os.path.basename` / `dirname` /
# `join` / `splitext` / `normpath` / `isabs` / `abspath` /
# `exists` all hasattr False, ipaddress.ip_address constructor
# returns an integer handle instead of an IPv4Address /
# IPv6Address instance, ipaddress.ip_network constructor
# returns an integer handle instead of a network instance,
# .is_loopback returns None on every address, .prefixlen /
# .num_addresses return None, address-in-network membership
# returns False even for valid containing networks) are covered
# in the matching spec fixture
# `lang_ospath_ipaddress_instance_silent`.
import posixpath
import mimetypes
import ipaddress


_ledger: list[int] = []

# 1) posixpath — basename / dirname / split / splitext
assert posixpath.basename("/foo/bar/baz.txt") == "baz.txt"; _ledger.append(1)
assert posixpath.dirname("/foo/bar/baz.txt") == "/foo/bar"; _ledger.append(1)
assert posixpath.split("/a/b/c") == ("/a/b", "c"); _ledger.append(1)
assert posixpath.splitext("/foo/bar/baz.txt") == ("/foo/bar/baz", ".txt"); _ledger.append(1)
assert posixpath.splitext("/foo/bar/baz") == ("/foo/bar/baz", ""); _ledger.append(1)

# 2) posixpath — join / normpath / isabs
assert posixpath.join("a", "b", "c") == "a/b/c"; _ledger.append(1)
assert posixpath.normpath("/a/b/../c") == "/a/c"; _ledger.append(1)
assert posixpath.isabs("/a") == True; _ledger.append(1)
assert posixpath.isabs("a") == False; _ledger.append(1)

# 3) posixpath — module hasattr surface
assert hasattr(posixpath, "basename") == True; _ledger.append(1)
assert hasattr(posixpath, "dirname") == True; _ledger.append(1)
assert hasattr(posixpath, "join") == True; _ledger.append(1)
assert hasattr(posixpath, "splitext") == True; _ledger.append(1)
assert hasattr(posixpath, "normpath") == True; _ledger.append(1)
assert hasattr(posixpath, "isabs") == True; _ledger.append(1)
assert hasattr(posixpath, "split") == True; _ledger.append(1)

# 4) mimetypes — guess_type against canonical extensions
assert mimetypes.guess_type("foo.html") == ("text/html", None); _ledger.append(1)
assert mimetypes.guess_type("foo.txt") == ("text/plain", None); _ledger.append(1)
assert mimetypes.guess_type("foo.jpg") == ("image/jpeg", None); _ledger.append(1)
assert mimetypes.guess_type("foo.png") == ("image/png", None); _ledger.append(1)
assert mimetypes.guess_type("foo.unknown_xyz") == (None, None); _ledger.append(1)

# 5) mimetypes — module hasattr surface
assert hasattr(mimetypes, "guess_type") == True; _ledger.append(1)
assert hasattr(mimetypes, "guess_extension") == True; _ledger.append(1)
assert hasattr(mimetypes, "init") == True; _ledger.append(1)
assert hasattr(mimetypes, "add_type") == True; _ledger.append(1)

# 6) ipaddress — class identifier hasattr surface
assert hasattr(ipaddress, "ip_address") == True; _ledger.append(1)
assert hasattr(ipaddress, "ip_network") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv4Address") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv6Address") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv4Network") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv6Network") == True; _ledger.append(1)

# 7) ipaddress — `.version` integer property
assert ipaddress.ip_address("192.168.1.1").version == 4; _ledger.append(1)
assert ipaddress.ip_address("::1").version == 6; _ledger.append(1)
assert ipaddress.ip_address("10.0.0.1").version == 4; _ledger.append(1)

# 8) ipaddress — RFC1918 `.is_private` property
assert ipaddress.ip_address("192.168.1.1").is_private == True; _ledger.append(1)

# NB: entire `os.path` attribute surface absent — `os.path.basename`
# / `dirname` / `join` / `splitext` / `normpath` / `isabs` /
# `abspath` / `exists` all hasattr False, ipaddress.ip_address
# constructor returns an integer handle instead of an
# IPv4Address / IPv6Address instance, ipaddress.ip_network
# constructor returns an integer handle, .is_loopback returns
# None on every address, .prefixlen / .num_addresses return
# None, address-in-network membership returns False — all
# DIVERGE on mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_posixpath_mimetypes_ipaddress_value_ops {sum(_ledger)} asserts")
