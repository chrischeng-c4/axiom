# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipaddr_unit_test__test_get_subnets3"
# subject = "cpython.test_ipaddress.IpaddrUnitTest.testGetSubnets3"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ipaddress.py::IpaddrUnitTest::testGetSubnets3
"""Auto-ported test: IpaddrUnitTest::testGetSubnets3 (CPython 3.12 oracle)."""


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
subnets = [str(x) for x in self_ipv4_network.subnets(8)]

assert subnets[:3] == ['1.2.3.0/32', '1.2.3.1/32', '1.2.3.2/32']

assert subnets[-3:] == ['1.2.3.253/32', '1.2.3.254/32', '1.2.3.255/32']

assert len(subnets) == 256
ipv6_network = ipaddress.IPv6Network('2001:658:22a:cafe::/120')
subnets = [str(x) for x in ipv6_network.subnets(8)]

assert subnets[:3] == ['2001:658:22a:cafe::/128', '2001:658:22a:cafe::1/128', '2001:658:22a:cafe::2/128']

assert subnets[-3:] == ['2001:658:22a:cafe::fd/128', '2001:658:22a:cafe::fe/128', '2001:658:22a:cafe::ff/128']

assert len(subnets) == 256
print("IpaddrUnitTest::testGetSubnets3: ok")
