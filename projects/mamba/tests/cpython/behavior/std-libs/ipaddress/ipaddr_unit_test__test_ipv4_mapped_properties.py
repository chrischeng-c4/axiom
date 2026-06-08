# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipaddr_unit_test__test_ipv4_mapped_properties"
# subject = "cpython.test_ipaddress.IpaddrUnitTest.testIpv4MappedProperties"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ipaddress.py::IpaddrUnitTest::testIpv4MappedProperties
"""Auto-ported test: IpaddrUnitTest::testIpv4MappedProperties (CPython 3.12 oracle)."""


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
for addr4 in ('178.62.3.251', '169.254.169.254', '127.0.0.1', '224.0.0.1', '192.168.0.1', '0.0.0.0', '100.64.0.1'):
    ipv4 = ipaddress.IPv4Address(addr4)
    ipv6 = ipaddress.IPv6Address(f'::ffff:{addr4}')

    assert ipv4.is_global == ipv6.is_global

    assert ipv4.is_private == ipv6.is_private

    assert ipv4.is_reserved == ipv6.is_reserved

    assert ipv4.is_multicast == ipv6.is_multicast

    assert ipv4.is_unspecified == ipv6.is_unspecified

    assert ipv4.is_link_local == ipv6.is_link_local

    assert ipv4.is_loopback == ipv6.is_loopback
print("IpaddrUnitTest::testIpv4MappedProperties: ok")
