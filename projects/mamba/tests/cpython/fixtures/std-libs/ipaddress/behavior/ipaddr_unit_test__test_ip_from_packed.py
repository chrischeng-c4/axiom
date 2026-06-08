# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipaddr_unit_test__test_ip_from_packed"
# subject = "cpython.test_ipaddress.IpaddrUnitTest.testIpFromPacked"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ipaddress.py::IpaddrUnitTest::testIpFromPacked
"""Auto-ported test: IpaddrUnitTest::testIpFromPacked (CPython 3.12 oracle)."""


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
address = ipaddress.ip_address

assert self_ipv4_interface._ip == ipaddress.ip_interface(b'\x01\x02\x03\x04')._ip

assert address('255.254.253.252') == address(b'\xff\xfe\xfd\xfc')

assert self_ipv6_interface.ip == ipaddress.ip_interface(b' \x01\x06X\x02*\xca\xfe\x02\x00\x00\x00\x00\x00\x00\x01').ip

assert address('ffff:2:3:4:ffff::') == address(b'\xff\xff\x00\x02\x00\x03\x00\x04' + b'\xff\xff' + b'\x00' * 6)

assert address('::') == address(b'\x00' * 16)
print("IpaddrUnitTest::testIpFromPacked: ok")
