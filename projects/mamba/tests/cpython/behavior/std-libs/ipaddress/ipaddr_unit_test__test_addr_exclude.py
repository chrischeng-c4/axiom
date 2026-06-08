# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipaddr_unit_test__test_addr_exclude"
# subject = "cpython.test_ipaddress.IpaddrUnitTest.testAddrExclude"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ipaddress.py::IpaddrUnitTest::testAddrExclude
"""Auto-ported test: IpaddrUnitTest::testAddrExclude (CPython 3.12 oracle)."""


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
addr1 = ipaddress.ip_network('10.1.1.0/24')
addr2 = ipaddress.ip_network('10.1.1.0/26')
addr3 = ipaddress.ip_network('10.2.1.0/24')
addr4 = ipaddress.ip_address('10.1.1.0')
addr5 = ipaddress.ip_network('2001:db8::0/32')
addr6 = ipaddress.ip_network('10.1.1.5/32')

assert sorted(list(addr1.address_exclude(addr2))) == [ipaddress.ip_network('10.1.1.64/26'), ipaddress.ip_network('10.1.1.128/25')]

try:
    list(addr1.address_exclude(addr3))
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    list(addr1.address_exclude(addr4))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    list(addr1.address_exclude(addr5))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert list(addr1.address_exclude(addr1)) == []

assert sorted(list(addr1.address_exclude(addr6))) == [ipaddress.ip_network('10.1.1.0/30'), ipaddress.ip_network('10.1.1.4/32'), ipaddress.ip_network('10.1.1.6/31'), ipaddress.ip_network('10.1.1.8/29'), ipaddress.ip_network('10.1.1.16/28'), ipaddress.ip_network('10.1.1.32/27'), ipaddress.ip_network('10.1.1.64/26'), ipaddress.ip_network('10.1.1.128/25')]
print("IpaddrUnitTest::testAddrExclude: ok")
