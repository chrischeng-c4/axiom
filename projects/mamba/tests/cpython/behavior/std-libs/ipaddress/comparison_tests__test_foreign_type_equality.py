# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "comparison_tests__test_foreign_type_equality"
# subject = "cpython.test_ipaddress.ComparisonTests.test_foreign_type_equality"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ipaddress.py::ComparisonTests::test_foreign_type_equality
"""Auto-ported test: ComparisonTests::test_foreign_type_equality (CPython 3.12 oracle)."""


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
other = object()
for obj in objects_with_scoped:

    assert obj != other

    assert not obj == other

    assert obj.__eq__(other) == NotImplemented

    assert obj.__ne__(other) == NotImplemented
print("ComparisonTests::test_foreign_type_equality: ok")
