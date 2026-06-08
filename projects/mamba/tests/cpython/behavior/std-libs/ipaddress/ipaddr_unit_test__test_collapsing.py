# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipaddr_unit_test__test_collapsing"
# subject = "cpython.test_ipaddress.IpaddrUnitTest.testCollapsing"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ipaddress.py::IpaddrUnitTest::testCollapsing
"""Auto-ported test: IpaddrUnitTest::testCollapsing (CPython 3.12 oracle)."""


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
ip1 = ipaddress.IPv4Address('1.1.1.0')
ip2 = ipaddress.IPv4Address('1.1.1.1')
ip3 = ipaddress.IPv4Address('1.1.1.2')
ip4 = ipaddress.IPv4Address('1.1.1.3')
ip5 = ipaddress.IPv4Address('1.1.1.4')
ip6 = ipaddress.IPv4Address('1.1.1.0')
collapsed = ipaddress.collapse_addresses([ip1, ip2, ip3, ip4, ip5, ip6])

assert list(collapsed) == [ipaddress.IPv4Network('1.1.1.0/30'), ipaddress.IPv4Network('1.1.1.4/32')]
ip1 = ipaddress.IPv4Address('1.1.1.0')
ip2 = ipaddress.IPv4Address('1.1.1.1')
ip3 = ipaddress.IPv4Address('1.1.1.2')
ip4 = ipaddress.IPv4Address('1.1.1.3')
collapsed = ipaddress.collapse_addresses([ip1, ip2, ip3, ip4])

assert list(collapsed) == [ipaddress.IPv4Network('1.1.1.0/30')]
ip1 = ipaddress.IPv4Network('1.1.0.0/24')
ip2 = ipaddress.IPv4Network('1.1.1.0/24')
ip3 = ipaddress.IPv4Network('1.1.2.0/24')
ip4 = ipaddress.IPv4Network('1.1.3.0/24')
ip5 = ipaddress.IPv4Network('1.1.4.0/24')
ip6 = ipaddress.IPv4Network('1.1.0.0/22')
collapsed = ipaddress.collapse_addresses([ip1, ip2, ip3, ip4, ip5, ip6])

assert list(collapsed) == [ipaddress.IPv4Network('1.1.0.0/22'), ipaddress.IPv4Network('1.1.4.0/24')]
collapsed = ipaddress.collapse_addresses([ip1, ip2])

assert list(collapsed) == [ipaddress.IPv4Network('1.1.0.0/23')]
ip_same1 = ip_same2 = ipaddress.IPv4Network('1.1.1.1/32')

assert list(ipaddress.collapse_addresses([ip_same1, ip_same2])) == [ip_same1]
ip_same1 = ip_same2 = ipaddress.IPv4Address('1.1.1.1')

assert list(ipaddress.collapse_addresses([ip_same1, ip_same2])) == [ipaddress.ip_network('1.1.1.1/32')]
ip1 = ipaddress.IPv6Network('2001::/100')
ip2 = ipaddress.IPv6Network('2001::/120')
ip3 = ipaddress.IPv6Network('2001::/96')
collapsed = ipaddress.collapse_addresses([ip1, ip2, ip3])

assert list(collapsed) == [ip3]
ip1 = ipaddress.IPv6Network('2001::%scope/100')
ip2 = ipaddress.IPv6Network('2001::%scope/120')
ip3 = ipaddress.IPv6Network('2001::%scope/96')
collapsed = ipaddress.collapse_addresses([ip1, ip2, ip3])

assert list(collapsed) == [ip3]
addr_tuples = [(ipaddress.ip_address('1.1.1.1'), ipaddress.ip_address('::1')), (ipaddress.IPv4Network('1.1.0.0/24'), ipaddress.IPv6Network('2001::/120')), (ipaddress.IPv4Network('1.1.0.0/32'), ipaddress.IPv6Network('2001::/128'))]
for ip1, ip2 in addr_tuples:

    try:
        ipaddress.collapse_addresses([ip1, ip2])
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
addr_tuples = [(ipaddress.ip_address('1.1.1.1'), ipaddress.ip_address('::1%scope')), (ipaddress.IPv4Network('1.1.0.0/24'), ipaddress.IPv6Network('2001::%scope/120')), (ipaddress.IPv4Network('1.1.0.0/32'), ipaddress.IPv6Network('2001::%scope/128'))]
for ip1, ip2 in addr_tuples:

    try:
        ipaddress.collapse_addresses([ip1, ip2])
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("IpaddrUnitTest::testCollapsing: ok")
