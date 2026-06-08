# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipaddr_unit_test__test_compress_i_pv6_address"
# subject = "cpython.test_ipaddress.IpaddrUnitTest.testCompressIPv6Address"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ipaddress.py::IpaddrUnitTest::testCompressIPv6Address
"""Auto-ported test: IpaddrUnitTest::testCompressIPv6Address (CPython 3.12 oracle)."""


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
test_addresses = {'1:2:3:4:5:6:7:8': '1:2:3:4:5:6:7:8/128', '2001:0:0:4:0:0:0:8': '2001:0:0:4::8/128', '2001:0:0:4:5:6:7:8': '2001::4:5:6:7:8/128', '2001:0:3:4:5:6:7:8': '2001:0:3:4:5:6:7:8/128', '0:0:3:0:0:0:0:ffff': '0:0:3::ffff/128', '0:0:0:4:0:0:0:ffff': '::4:0:0:0:ffff/128', '0:0:0:0:5:0:0:ffff': '::5:0:0:ffff/128', '1:0:0:4:0:0:7:8': '1::4:0:0:7:8/128', '0:0:0:0:0:0:0:0': '::/128', '0:0:0:0:0:0:0:0/0': '::/0', '0:0:0:0:0:0:0:1': '::1/128', '2001:0658:022a:cafe:0000:0000:0000:0000/66': '2001:658:22a:cafe::/66', '::1.2.3.4': '::102:304/128', '1:2:3:4:5:ffff:1.2.3.4': '1:2:3:4:5:ffff:102:304/128', '::7:6:5:4:3:2:1': '0:7:6:5:4:3:2:1/128', '::7:6:5:4:3:2:0': '0:7:6:5:4:3:2:0/128', '7:6:5:4:3:2:1::': '7:6:5:4:3:2:1:0/128', '0:6:5:4:3:2:1::': '0:6:5:4:3:2:1:0/128', '0000:0000:0000:0000:0000:0000:255.255.255.255': '::ffff:ffff/128', '0000:0000:0000:0000:0000:ffff:255.255.255.255': '::ffff:255.255.255.255/128', 'ffff:ffff:ffff:ffff:ffff:ffff:255.255.255.255': 'ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff/128'}
for uncompressed, compressed in list(test_addresses.items()):

    assert compressed == str(ipaddress.IPv6Interface(uncompressed))
print("IpaddrUnitTest::testCompressIPv6Address: ok")
