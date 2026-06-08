# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipaddr_unit_test__test_get_subnet_for_single128"
# subject = "cpython.test_ipaddress.IpaddrUnitTest.testGetSubnetForSingle128"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ipaddress.py::IpaddrUnitTest::testGetSubnetForSingle128
"""Auto-ported test: IpaddrUnitTest::testGetSubnetForSingle128 (CPython 3.12 oracle)."""


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
ip = ipaddress.IPv6Network('::1/128')
subnets1 = [str(x) for x in ip.subnets()]
subnets2 = [str(x) for x in ip.subnets(2)]

assert subnets1 == ['::1/128']

assert subnets1 == subnets2
ip_scoped = ipaddress.IPv6Network('::1%scope/128')
subnets1 = [str(x) for x in ip_scoped.subnets()]
subnets2 = [str(x) for x in ip_scoped.subnets(2)]

assert subnets1 == ['::1%scope/128']

assert subnets1 == subnets2
print("IpaddrUnitTest::testGetSubnetForSingle128: ok")
