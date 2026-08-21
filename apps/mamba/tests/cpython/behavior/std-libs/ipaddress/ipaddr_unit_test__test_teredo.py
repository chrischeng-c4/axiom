# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipaddr_unit_test__test_teredo"
# subject = "cpython.test_ipaddress.IpaddrUnitTest.testTeredo"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ipaddress.py::IpaddrUnitTest::testTeredo
"""Auto-ported test: IpaddrUnitTest::testTeredo (CPython 3.12 oracle)."""


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
server = ipaddress.IPv4Address('65.54.227.120')
client = ipaddress.IPv4Address('192.0.2.45')
teredo_addr = '2001:0000:4136:e378:8000:63bf:3fff:fdd2'

assert (server, client) == ipaddress.ip_address(teredo_addr).teredo
bad_addr = '2000::4136:e378:8000:63bf:3fff:fdd2'

assert not ipaddress.ip_address(bad_addr).teredo
bad_addr = '2001:0001:4136:e378:8000:63bf:3fff:fdd2'

assert not ipaddress.ip_address(bad_addr).teredo
teredo_addr = ipaddress.IPv6Address('2001:0:5ef5:79fd:0:59d:a0e5:ba1')

assert (ipaddress.IPv4Address('94.245.121.253'), ipaddress.IPv4Address('95.26.244.94')) == teredo_addr.teredo
print("IpaddrUnitTest::testTeredo: ok")
