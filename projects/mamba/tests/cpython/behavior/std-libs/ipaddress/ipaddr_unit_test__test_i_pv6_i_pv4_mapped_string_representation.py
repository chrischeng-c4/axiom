# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipaddr_unit_test__test_i_pv6_i_pv4_mapped_string_representation"
# subject = "cpython.test_ipaddress.IpaddrUnitTest.testIPv6IPv4MappedStringRepresentation"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ipaddress.py::IpaddrUnitTest::testIPv6IPv4MappedStringRepresentation
"""Auto-ported test: IpaddrUnitTest::testIPv6IPv4MappedStringRepresentation (CPython 3.12 oracle)."""


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
long_prefix = '0000:0000:0000:0000:0000:ffff:'
short_prefix = '::ffff:'
ipv4 = '1.2.3.4'
ipv6_ipv4_str = short_prefix + ipv4
ipv6_ipv4_addr = ipaddress.IPv6Address(ipv6_ipv4_str)
ipv6_ipv4_iface = ipaddress.IPv6Interface(ipv6_ipv4_str)

assert str(ipv6_ipv4_addr) == ipv6_ipv4_str

assert ipv6_ipv4_addr.exploded == long_prefix + ipv4

assert str(ipv6_ipv4_iface.ip) == ipv6_ipv4_str
print("IpaddrUnitTest::testIPv6IPv4MappedStringRepresentation: ok")
