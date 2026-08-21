# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipaddr_unit_test__test_zero_netmask"
# subject = "cpython.test_ipaddress.IpaddrUnitTest.testZeroNetmask"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ipaddress.py::IpaddrUnitTest::testZeroNetmask
"""Auto-ported test: IpaddrUnitTest::testZeroNetmask (CPython 3.12 oracle)."""


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
self_ipv4_address = ipaddress.IPv4Address('1.2.3.4')
self_ipv4_interface = ipaddress.IPv4Interface('1.2.3.4/24')
self_ipv4_network = ipaddress.IPv4Network('1.2.3.0/24')
self_ipv6_address = ipaddress.IPv6Interface('2001:658:22a:cafe:200:0:0:1')
self_ipv6_interface = ipaddress.IPv6Interface('2001:658:22a:cafe:200:0:0:1/64')
self_ipv6_network = ipaddress.IPv6Network('2001:658:22a:cafe::/64')
self_ipv6_scoped_address = ipaddress.IPv6Interface('2001:658:22a:cafe:200:0:0:1%scope')
self_ipv6_scoped_interface = ipaddress.IPv6Interface('2001:658:22a:cafe:200:0:0:1%scope/64')
self_ipv6_scoped_network = ipaddress.IPv6Network('2001:658:22a:cafe::%scope/64')
self_ipv6_with_ipv4_part = ipaddress.IPv6Interface('::1.2.3.4')
ipv4_zero_netmask = ipaddress.IPv4Interface('1.2.3.4/0')

assert int(ipv4_zero_netmask.network.netmask) == 0

assert ipv4_zero_netmask._prefix_from_prefix_string('0') == 0
ipv6_zero_netmask = ipaddress.IPv6Interface('::1/0')

assert int(ipv6_zero_netmask.network.netmask) == 0

assert ipv6_zero_netmask._prefix_from_prefix_string('0') == 0
ipv6_scoped_zero_netmask = ipaddress.IPv6Interface('::1%scope/0')

assert int(ipv6_scoped_zero_netmask.network.netmask) == 0

assert ipv6_scoped_zero_netmask._prefix_from_prefix_string('0') == 0
print("IpaddrUnitTest::testZeroNetmask: ok")
