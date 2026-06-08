# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "comparison_tests__test_mixed_type_key"
# subject = "cpython.test_ipaddress.ComparisonTests.test_mixed_type_key"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ipaddress.py::ComparisonTests::test_mixed_type_key
"""Auto-ported test: ComparisonTests::test_mixed_type_key (CPython 3.12 oracle)."""


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
v4_ordered = [v4addr, v4net, v4intf]
v6_ordered = [v6addr, v6net, v6intf]
v6_scoped_ordered = [v6addr_scoped, v6net_scoped, v6intf_scoped]

assert v4_ordered == sorted(v4_objects, key=ipaddress.get_mixed_type_key)

assert v6_ordered == sorted(v6_objects, key=ipaddress.get_mixed_type_key)

assert v6_scoped_ordered == sorted(v6_scoped_objects, key=ipaddress.get_mixed_type_key)

assert v4_ordered + v6_scoped_ordered == sorted(v4_objects + v6_scoped_objects, key=ipaddress.get_mixed_type_key)

assert NotImplemented == ipaddress.get_mixed_type_key(object)
print("ComparisonTests::test_mixed_type_key: ok")
