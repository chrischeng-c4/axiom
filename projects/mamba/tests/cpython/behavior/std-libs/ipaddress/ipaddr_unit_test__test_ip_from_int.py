# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipaddr_unit_test__test_ip_from_int"
# subject = "cpython.test_ipaddress.IpaddrUnitTest.testIpFromInt"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ipaddress.py::IpaddrUnitTest::testIpFromInt
"""Auto-ported test: IpaddrUnitTest::testIpFromInt (CPython 3.12 oracle)."""


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

assert self_ipv4_interface._ip == ipaddress.IPv4Interface(16909060)._ip
ipv4 = ipaddress.ip_network('1.2.3.4')
ipv6 = ipaddress.ip_network('2001:658:22a:cafe:200:0:0:1')
ipv6_scoped = ipaddress.ip_network('2001:658:22a:cafe:200:0:0:1%scope')

assert ipv4 == ipaddress.ip_network(int(ipv4.network_address))

assert ipv6 == ipaddress.ip_network(int(ipv6.network_address))

assert ipv6_scoped != ipaddress.ip_network(int(ipv6_scoped.network_address))
v6_int = 42540616829182469433547762482097946625

assert self_ipv6_interface._ip == ipaddress.IPv6Interface(v6_int)._ip

assert self_ipv6_scoped_interface._ip == ipaddress.IPv6Interface(v6_int)._ip

assert ipaddress.ip_network(self_ipv4_address._ip).version == 4

assert ipaddress.ip_network(self_ipv6_address._ip).version == 6

assert ipaddress.ip_network(self_ipv6_scoped_address._ip).version == 6
print("IpaddrUnitTest::testIpFromInt: ok")
