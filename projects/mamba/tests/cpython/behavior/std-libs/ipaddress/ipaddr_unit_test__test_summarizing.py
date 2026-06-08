# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipaddr_unit_test__test_summarizing"
# subject = "cpython.test_ipaddress.IpaddrUnitTest.testSummarizing"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ipaddress.py::IpaddrUnitTest::testSummarizing
"""Auto-ported test: IpaddrUnitTest::testSummarizing (CPython 3.12 oracle)."""


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
summarize = ipaddress.summarize_address_range
ip1 = ipaddress.ip_address('1.1.1.0')
ip2 = ipaddress.ip_address('1.1.1.255')

class IPv7Address(ipaddress.IPv6Address):

    @property
    def version(self):
        return 7
ip_invalid1 = IPv7Address('::1')
ip_invalid2 = IPv7Address('::1')

try:
    list(summarize(ip_invalid1, ip_invalid2))
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    list(summarize(ip1, ipaddress.IPv6Address('::1')))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    list(summarize(ip1, ipaddress.IPv6Address('::1%scope')))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert list(summarize(ip1, ip2))[0] == ipaddress.ip_network('1.1.1.0/24')
ip2 = ipaddress.ip_address('1.1.1.8')

assert list(summarize(ip1, ip2)) == [ipaddress.ip_network('1.1.1.0/29'), ipaddress.ip_network('1.1.1.8')]
ip1 = ipaddress.IPv4Address(0)
ip2 = ipaddress.IPv4Address(ipaddress.IPv4Address._ALL_ONES)

assert [ipaddress.IPv4Network('0.0.0.0/0')] == list(summarize(ip1, ip2))
ip1 = ipaddress.ip_address('1::')
ip2 = ipaddress.ip_address('1:ffff:ffff:ffff:ffff:ffff:ffff:ffff')

assert list(summarize(ip1, ip2))[0] == ipaddress.ip_network('1::/16')
ip2 = ipaddress.ip_address('2::')

assert list(summarize(ip1, ip2)) == [ipaddress.ip_network('1::/16'), ipaddress.ip_network('2::/128')]
ip1 = ipaddress.ip_address('1::%scope')
ip2 = ipaddress.ip_address('1:ffff:ffff:ffff:ffff:ffff:ffff:ffff%scope')

assert list(summarize(ip1, ip2))[0] == ipaddress.ip_network('1::/16')
ip2 = ipaddress.ip_address('2::%scope')

assert list(summarize(ip1, ip2)) == [ipaddress.ip_network('1::/16'), ipaddress.ip_network('2::/128')]

try:
    list(summarize(ipaddress.ip_address('1.1.1.0'), ipaddress.ip_address('1.1.0.0')))
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    list(summarize(ipaddress.ip_network('1.1.1.0'), ipaddress.ip_network('1.1.0.0')))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    list(summarize(ipaddress.ip_network('1.1.1.0'), ipaddress.ip_network('1.1.0.0')))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    list(summarize(ipaddress.ip_address('::'), ipaddress.ip_network('1.1.0.0')))
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("IpaddrUnitTest::testSummarizing: ok")
