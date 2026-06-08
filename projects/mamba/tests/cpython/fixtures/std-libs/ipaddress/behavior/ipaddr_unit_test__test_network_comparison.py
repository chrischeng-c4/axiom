# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipaddr_unit_test__test_network_comparison"
# subject = "cpython.test_ipaddress.IpaddrUnitTest.testNetworkComparison"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ipaddress.py::IpaddrUnitTest::testNetworkComparison
"""Auto-ported test: IpaddrUnitTest::testNetworkComparison (CPython 3.12 oracle)."""


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
ip1 = ipaddress.IPv4Network('1.1.1.0/24')
ip2 = ipaddress.IPv4Network('1.1.1.0/32')
ip3 = ipaddress.IPv4Network('1.1.2.0/24')

assert ip1 < ip3

assert ip3 > ip2

assert ip1.compare_networks(ip1) == 0

assert ip1.compare_networks(ip2) == -1

assert ip2.compare_networks(ip1) == 1

assert ip1.compare_networks(ip3) == -1

assert ip3.compare_networks(ip1) == 1

assert ip1._get_networks_key() < ip3._get_networks_key()
ip1 = ipaddress.IPv6Network('2001:2000::/96')
ip2 = ipaddress.IPv6Network('2001:2001::/96')
ip3 = ipaddress.IPv6Network('2001:ffff:2000::/96')

assert ip1 < ip3

assert ip3 > ip2

assert ip1.compare_networks(ip3) == -1

assert ip1._get_networks_key() < ip3._get_networks_key()

try:
    self_ipv4_network.compare_networks(self_ipv6_network)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
ipv6 = ipaddress.IPv6Interface('::/0')
ipv4 = ipaddress.IPv4Interface('0.0.0.0/0')

try:
    ipv4.__lt__(ipv6)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    ipv4.__gt__(ipv6)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    ipv6.__lt__(ipv4)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    ipv6.__gt__(ipv4)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
ip1 = ipaddress.ip_network('10.1.2.128/25')

assert not ip1 < ip1

assert not ip1 > ip1
ip2 = ipaddress.ip_network('10.1.3.0/24')

assert ip1 < ip2

assert not ip2 < ip1

assert not ip1 > ip2

assert ip2 > ip1
ip3 = ipaddress.ip_network('10.1.3.0/25')

assert ip2 < ip3

assert not ip3 < ip2

assert not ip2 > ip3

assert ip3 > ip2
ip1 = ipaddress.ip_network('10.10.10.0/31')
ip2 = ipaddress.ip_network('10.10.10.0')
ip3 = ipaddress.ip_network('10.10.10.2/31')
ip4 = ipaddress.ip_network('10.10.10.2')
sorted = [ip1, ip2, ip3, ip4]
unsorted = [ip2, ip4, ip1, ip3]
unsorted.sort()

assert sorted == unsorted
unsorted = [ip4, ip1, ip3, ip2]
unsorted.sort()

assert sorted == unsorted

assert ip1.__lt__(ipaddress.ip_address('10.10.10.0')) is NotImplemented

assert ip2.__lt__(ipaddress.ip_address('10.10.10.0')) is NotImplemented

assert ipaddress.ip_network('1.1.1.1') <= ipaddress.ip_network('1.1.1.1')

assert ipaddress.ip_network('1.1.1.1') <= ipaddress.ip_network('1.1.1.2')

assert not ipaddress.ip_network('1.1.1.2') <= ipaddress.ip_network('1.1.1.1')

assert ipaddress.ip_network('::1') <= ipaddress.ip_network('::1')

assert ipaddress.ip_network('::1') <= ipaddress.ip_network('::2')

assert not ipaddress.ip_network('::2') <= ipaddress.ip_network('::1')
print("IpaddrUnitTest::testNetworkComparison: ok")
