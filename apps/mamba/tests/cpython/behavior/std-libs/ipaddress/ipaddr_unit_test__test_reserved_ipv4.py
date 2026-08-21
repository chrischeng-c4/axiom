# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipaddr_unit_test__test_reserved_ipv4"
# subject = "cpython.test_ipaddress.IpaddrUnitTest.testReservedIpv4"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ipaddress.py::IpaddrUnitTest::testReservedIpv4
"""Auto-ported test: IpaddrUnitTest::testReservedIpv4 (CPython 3.12 oracle)."""


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

assert True == ipaddress.ip_interface('224.1.1.1/31').is_multicast

assert False == ipaddress.ip_network('240.0.0.0').is_multicast

assert True == ipaddress.ip_network('240.0.0.0').is_reserved

assert True == ipaddress.ip_interface('192.168.1.1/17').is_private

assert False == ipaddress.ip_network('192.169.0.0').is_private

assert True == ipaddress.ip_network('10.255.255.255').is_private

assert False == ipaddress.ip_network('11.0.0.0').is_private

assert False == ipaddress.ip_network('11.0.0.0').is_reserved

assert True == ipaddress.ip_network('172.31.255.255').is_private

assert False == ipaddress.ip_network('172.32.0.0').is_private

assert True == ipaddress.ip_network('169.254.1.0/24').is_link_local

assert True == ipaddress.ip_interface('169.254.100.200/24').is_link_local

assert False == ipaddress.ip_interface('169.255.100.200/24').is_link_local

assert True == ipaddress.ip_network('127.100.200.254/32').is_loopback

assert True == ipaddress.ip_network('127.42.0.0/16').is_loopback

assert False == ipaddress.ip_network('128.0.0.0').is_loopback

assert False == ipaddress.ip_network('100.64.0.0/10').is_private

assert False == ipaddress.ip_network('100.64.0.0/10').is_global

assert True == ipaddress.ip_network('192.0.2.128/25').is_private

assert True == ipaddress.ip_network('192.0.3.0/24').is_global

assert True == ipaddress.ip_address('0.0.0.0').is_unspecified

assert True == ipaddress.ip_address('224.1.1.1').is_multicast

assert False == ipaddress.ip_address('240.0.0.0').is_multicast

assert True == ipaddress.ip_address('240.0.0.1').is_reserved

assert False == ipaddress.ip_address('239.255.255.255').is_reserved

assert True == ipaddress.ip_address('192.168.1.1').is_private

assert False == ipaddress.ip_address('192.169.0.0').is_private

assert True == ipaddress.ip_address('10.255.255.255').is_private

assert False == ipaddress.ip_address('11.0.0.0').is_private

assert True == ipaddress.ip_address('172.31.255.255').is_private

assert False == ipaddress.ip_address('172.32.0.0').is_private

assert not ipaddress.ip_address('192.0.0.0').is_global

assert ipaddress.ip_address('192.0.0.9').is_global

assert ipaddress.ip_address('192.0.0.10').is_global

assert not ipaddress.ip_address('192.0.0.255').is_global

assert True == ipaddress.ip_address('169.254.100.200').is_link_local

assert False == ipaddress.ip_address('169.255.100.200').is_link_local

assert ipaddress.ip_address('192.0.7.1').is_global

assert not ipaddress.ip_address('203.0.113.1').is_global

assert True == ipaddress.ip_address('127.100.200.254').is_loopback

assert True == ipaddress.ip_address('127.42.0.0').is_loopback

assert False == ipaddress.ip_address('128.0.0.0').is_loopback

assert True == ipaddress.ip_network('0.0.0.0').is_unspecified
print("IpaddrUnitTest::testReservedIpv4: ok")
