# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipaddr_unit_test__test_reserved_ipv6"
# subject = "cpython.test_ipaddress.IpaddrUnitTest.testReservedIpv6"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ipaddress.py::IpaddrUnitTest::testReservedIpv6
"""Auto-ported test: IpaddrUnitTest::testReservedIpv6 (CPython 3.12 oracle)."""


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

assert True == ipaddress.ip_network('ffff::').is_multicast

assert True == ipaddress.ip_network(2 ** 128 - 1).is_multicast

assert True == ipaddress.ip_network('ff00::').is_multicast

assert False == ipaddress.ip_network('fdff::').is_multicast

assert True == ipaddress.ip_network('fecf::').is_site_local

assert True == ipaddress.ip_network('feff:ffff:ffff:ffff::').is_site_local

assert False == ipaddress.ip_network('fbf:ffff::').is_site_local

assert False == ipaddress.ip_network('ff00::').is_site_local

assert True == ipaddress.ip_network('fc00::').is_private

assert True == ipaddress.ip_network('fc00:ffff:ffff:ffff::').is_private

assert False == ipaddress.ip_network('fbff:ffff::').is_private

assert False == ipaddress.ip_network('fe00::').is_private

assert True == ipaddress.ip_network('fea0::').is_link_local

assert True == ipaddress.ip_network('febf:ffff::').is_link_local

assert False == ipaddress.ip_network('fe7f:ffff::').is_link_local

assert False == ipaddress.ip_network('fec0::').is_link_local

assert True == ipaddress.ip_interface('0:0::0:01').is_loopback

assert False == ipaddress.ip_interface('::1/127').is_loopback

assert False == ipaddress.ip_network('::').is_loopback

assert False == ipaddress.ip_network('::2').is_loopback

assert True == ipaddress.ip_network('0::0').is_unspecified

assert False == ipaddress.ip_network('::1').is_unspecified

assert False == ipaddress.ip_network('::/127').is_unspecified

assert True == ipaddress.ip_network('2001::1/128').is_private

assert True == ipaddress.ip_network('200::1/128').is_global

assert True == ipaddress.ip_address('ffff::').is_multicast

assert True == ipaddress.ip_address(2 ** 128 - 1).is_multicast

assert True == ipaddress.ip_address('ff00::').is_multicast

assert False == ipaddress.ip_address('fdff::').is_multicast

assert True == ipaddress.ip_address('fecf::').is_site_local

assert True == ipaddress.ip_address('feff:ffff:ffff:ffff::').is_site_local

assert False == ipaddress.ip_address('fbf:ffff::').is_site_local

assert False == ipaddress.ip_address('ff00::').is_site_local

assert True == ipaddress.ip_address('fc00::').is_private

assert True == ipaddress.ip_address('fc00:ffff:ffff:ffff::').is_private

assert False == ipaddress.ip_address('fbff:ffff::').is_private

assert False == ipaddress.ip_address('fe00::').is_private

assert True == ipaddress.ip_address('fea0::').is_link_local

assert True == ipaddress.ip_address('febf:ffff::').is_link_local

assert False == ipaddress.ip_address('fe7f:ffff::').is_link_local

assert False == ipaddress.ip_address('fec0::').is_link_local

assert True == ipaddress.ip_address('0:0::0:01').is_loopback

assert True == ipaddress.ip_address('::1').is_loopback

assert False == ipaddress.ip_address('::2').is_loopback

assert True == ipaddress.ip_address('0::0').is_unspecified

assert False == ipaddress.ip_address('::1').is_unspecified

assert not ipaddress.ip_address('64:ff9b:1::').is_global

assert not ipaddress.ip_address('2001::').is_global

assert ipaddress.ip_address('2001:1::1').is_global

assert ipaddress.ip_address('2001:1::2').is_global

assert not ipaddress.ip_address('2001:2::').is_global

assert ipaddress.ip_address('2001:3::').is_global

assert not ipaddress.ip_address('2001:4::').is_global

assert ipaddress.ip_address('2001:4:112::').is_global

assert not ipaddress.ip_address('2001:10::').is_global

assert ipaddress.ip_address('2001:20::').is_global

assert ipaddress.ip_address('2001:30::').is_global

assert not ipaddress.ip_address('2001:40::').is_global

assert not ipaddress.ip_address('2002::').is_global

assert not ipaddress.ip_address('3fff::').is_global

assert True == ipaddress.ip_address('100::').is_reserved

assert True == ipaddress.ip_network('4000::1/128').is_reserved
print("IpaddrUnitTest::testReservedIpv6: ok")
