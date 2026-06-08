# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "comparison_tests__test_incompatible_versions"
# subject = "cpython.test_ipaddress.ComparisonTests.test_incompatible_versions"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ipaddress.py::ComparisonTests::test_incompatible_versions
"""Auto-ported test: ComparisonTests::test_incompatible_versions (CPython 3.12 oracle)."""


import copy
import unittest
import re
import contextlib
import operator
import pickle
import ipaddress
import weakref
from test.support import LARGEST, SMALLEST


'Unittest for ipaddress module.'


# --- test body ---
v4addr = ipaddress.IPv4Address(1)
v4net = ipaddress.IPv4Network(1)
v4intf = ipaddress.IPv4Interface(1)
v6addr = ipaddress.IPv6Address(1)
v6net = ipaddress.IPv6Network(1)
v6intf = ipaddress.IPv6Interface(1)
v6addr_scoped = ipaddress.IPv6Address('::1%scope')
v6net_scoped = ipaddress.IPv6Network('::1%scope')
v6intf_scoped = ipaddress.IPv6Interface('::1%scope')
v4_addresses = [v4addr, v4intf]
v4_objects = v4_addresses + [v4net]
v6_addresses = [v6addr, v6intf]
v6_objects = v6_addresses + [v6net]
v6_scoped_addresses = [v6addr_scoped, v6intf_scoped]
v6_scoped_objects = v6_scoped_addresses + [v6net_scoped]
objects = v4_objects + v6_objects
objects_with_scoped = objects + v6_scoped_objects
v4addr2 = ipaddress.IPv4Address(2)
v4net2 = ipaddress.IPv4Network(2)
v4intf2 = ipaddress.IPv4Interface(2)
v6addr2 = ipaddress.IPv6Address(2)
v6net2 = ipaddress.IPv6Network(2)
v6intf2 = ipaddress.IPv6Interface(2)
v6addr2_scoped = ipaddress.IPv6Address('::2%scope')
v6net2_scoped = ipaddress.IPv6Network('::2%scope')
v6intf2_scoped = ipaddress.IPv6Interface('::2%scope')
v4addr = ipaddress.ip_address('1.1.1.1')
v4net = ipaddress.ip_network('1.1.1.1')
v6addr = ipaddress.ip_address('::1')
v6net = ipaddress.ip_network('::1')
v6addr_scoped = ipaddress.ip_address('::1%scope')
v6net_scoped = ipaddress.ip_network('::1%scope')

try:
    v4addr.__lt__(v6addr)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    v4addr.__gt__(v6addr)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    v4net.__lt__(v6net)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    v4net.__gt__(v6net)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    v6addr.__lt__(v4addr)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    v6addr.__gt__(v4addr)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    v6net.__lt__(v4net)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    v6net.__gt__(v4net)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    v4addr.__lt__(v6addr_scoped)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    v4addr.__gt__(v6addr_scoped)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    v4net.__lt__(v6net_scoped)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    v4net.__gt__(v6net_scoped)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    v6addr_scoped.__lt__(v4addr)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    v6addr_scoped.__gt__(v4addr)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    v6net_scoped.__lt__(v4net)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    v6net_scoped.__gt__(v4net)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ComparisonTests::test_incompatible_versions: ok")
