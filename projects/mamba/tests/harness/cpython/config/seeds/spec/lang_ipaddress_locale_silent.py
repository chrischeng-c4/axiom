# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `type(ipaddress.ip_address('127.0.0.1
# ')).__name__ == 'IPv4Address'` (the documented "ip_address() of a
# dotted-quad returns an IPv4Address instance" — mamba returns 'int'
# — constructor degrades to a numeric handle), `str(ipaddress.ip_
# address('127.0.0.1')) == '127.0.0.1'` (the documented "str(IPv4
# Address) renders the canonical dotted-quad" — mamba returns a
# numeric handle string — str of the int handle), `int(ipaddress.ip_
# address('127.0.0.1')) == 2130706433` (the documented "int(IPv4
# Address) returns the 32-bit network-order integer" — mamba returns
# a value unrelated to the encoded address), `type(ipaddress.ip_
# address('::1')).__name__ == 'IPv6Address'` (the documented "ip_
# address() of a colon-form returns an IPv6Address instance" — mamba
# returns 'int' — constructor degrades to a numeric handle), `str(
# ipaddress.ip_address('::1')) == '::1'` (the documented "str(IPv6
# Address) renders the canonical compact-colon form" — mamba returns
# a numeric handle string), `ipaddress.ip_address('127.0.0.1') ==
# ipaddress.ip_address('127.0.0.1')` (the documented "ip_address
# equality compares the underlying address value" — mamba returns
# False — handle-identity equality), `str(ipaddress.ip_network('
# 192.168.0.0/24')) == '192.168.0.0/24'` (the documented "str(IPv4
# Network) renders the canonical network/prefix-length form" —
# mamba returns a numeric handle string), `type(ipaddress.ip_network(
# '192.168.0.0/24')).__name__ == 'IPv4Network'` (the documented
# "ip_network() of a v4 CIDR returns an IPv4Network instance" —
# mamba returns 'int' — constructor degrades to a numeric handle),
# `hasattr(locale, 'getpreferredencoding')` (the documented "locale
# exposes the getpreferredencoding helper" — mamba returns False),
# and `hasattr(gc, 'garbage')` (the documented "gc exposes the
# garbage list of uncollectable objects" — mamba returns False).
# Ten-pack pinned to atomic 303.
#
# Behavioral edges that CONFORM on mamba (pickle — hasattr dumps/
# loads/dump/load/Pickler/Unpickler/PickleError/PicklingError/
# UnpicklingError/HIGHEST_PROTOCOL/DEFAULT_PROTOCOL + dumps returns
# bytes + round-trip int/str/list/dict. numbers — hasattr Number/
# Complex/Real/Rational/Integral. ipaddress — hasattr ip_address/ip_
# network/IPv4Address/IPv6Address/IPv4Network/IPv6Network/Address
# ValueError/NetmaskValueError. locale — hasattr getlocale/setlocale
# /LC_ALL/LC_CTYPE/LC_NUMERIC/LC_TIME. gc — hasattr collect/disable/
# enable/isenabled/get_count/get_threshold/set_threshold/DEBUG_LEAK.
# atexit — hasattr register/unregister) are covered in the matching
# pass fixture `test_pickle_numbers_gc_atexit_value_ops`.
import ipaddress
import locale
import gc


_ledger: list[int] = []

# 1) type(ipaddress.ip_address('127.0.0.1')).__name__ == 'IPv4Address' — dotted-quad ctor returns IPv4Address
#    (mamba: returns 'int' — constructor degrades to numeric handle)
assert type(ipaddress.ip_address("127.0.0.1")).__name__ == "IPv4Address"; _ledger.append(1)

# 2) str(ipaddress.ip_address('127.0.0.1')) == '127.0.0.1' — canonical dotted-quad render
#    (mamba: returns a numeric handle string)
assert str(ipaddress.ip_address("127.0.0.1")) == "127.0.0.1"; _ledger.append(1)

# 3) int(ipaddress.ip_address('127.0.0.1')) == 2130706433 — 32-bit network-order integer
#    (mamba: returns a value unrelated to the encoded address)
assert int(ipaddress.ip_address("127.0.0.1")) == 2130706433; _ledger.append(1)

# 4) type(ipaddress.ip_address('::1')).__name__ == 'IPv6Address' — colon-form ctor returns IPv6Address
#    (mamba: returns 'int' — constructor degrades to numeric handle)
assert type(ipaddress.ip_address("::1")).__name__ == "IPv6Address"; _ledger.append(1)

# 5) str(ipaddress.ip_address('::1')) == '::1' — canonical compact-colon render
#    (mamba: returns a numeric handle string)
assert str(ipaddress.ip_address("::1")) == "::1"; _ledger.append(1)

# 6) ip_address eq across two equal addresses returns True — value equality
#    (mamba: returns False — handle-identity equality)
assert (ipaddress.ip_address("127.0.0.1") == ipaddress.ip_address("127.0.0.1")) == True; _ledger.append(1)

# 7) str(ipaddress.ip_network('192.168.0.0/24')) == '192.168.0.0/24' — canonical CIDR render
#    (mamba: returns a numeric handle string)
assert str(ipaddress.ip_network("192.168.0.0/24")) == "192.168.0.0/24"; _ledger.append(1)

# 8) type(ipaddress.ip_network('192.168.0.0/24')).__name__ == 'IPv4Network' — v4 CIDR ctor returns IPv4Network
#    (mamba: returns 'int' — constructor degrades to numeric handle)
assert type(ipaddress.ip_network("192.168.0.0/24")).__name__ == "IPv4Network"; _ledger.append(1)

# 9) hasattr(locale, 'getpreferredencoding') — getpreferredencoding helper
#    (mamba: returns False)
assert hasattr(locale, "getpreferredencoding") == True; _ledger.append(1)

# 10) hasattr(gc, 'garbage') — garbage list of uncollectable objects
#     (mamba: returns False)
assert hasattr(gc, "garbage") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_ipaddress_locale_silent {sum(_ledger)} asserts")
