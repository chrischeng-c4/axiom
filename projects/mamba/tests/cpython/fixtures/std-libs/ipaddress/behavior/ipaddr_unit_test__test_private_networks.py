# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipaddr_unit_test__test_private_networks"
# subject = "cpython.test_ipaddress.IpaddrUnitTest.testPrivateNetworks"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ipaddress.py::IpaddrUnitTest::testPrivateNetworks
"""Auto-ported test: IpaddrUnitTest::testPrivateNetworks (CPython 3.12 oracle)."""


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

assert False == ipaddress.ip_network('0.0.0.0/0').is_private

assert False == ipaddress.ip_network('1.0.0.0/8').is_private

assert True == ipaddress.ip_network('0.0.0.0/8').is_private

assert True == ipaddress.ip_network('10.0.0.0/8').is_private

assert True == ipaddress.ip_network('127.0.0.0/8').is_private

assert True == ipaddress.ip_network('169.254.0.0/16').is_private

assert True == ipaddress.ip_network('172.16.0.0/12').is_private

assert True == ipaddress.ip_network('192.0.0.0/29').is_private

assert False == ipaddress.ip_network('192.0.0.9/32').is_private

assert True == ipaddress.ip_network('192.0.0.170/31').is_private

assert True == ipaddress.ip_network('192.0.2.0/24').is_private

assert True == ipaddress.ip_network('192.168.0.0/16').is_private

assert True == ipaddress.ip_network('198.18.0.0/15').is_private

assert True == ipaddress.ip_network('198.51.100.0/24').is_private

assert True == ipaddress.ip_network('203.0.113.0/24').is_private

assert True == ipaddress.ip_network('240.0.0.0/4').is_private

assert True == ipaddress.ip_network('255.255.255.255/32').is_private

assert False == ipaddress.ip_network('::/0').is_private

assert False == ipaddress.ip_network('::ff/128').is_private

assert True == ipaddress.ip_network('::1/128').is_private

assert True == ipaddress.ip_network('::/128').is_private

assert True == ipaddress.ip_network('::ffff:0:0/96').is_private

assert True == ipaddress.ip_network('100::/64').is_private

assert True == ipaddress.ip_network('2001:2::/48').is_private

assert False == ipaddress.ip_network('2001:3::/48').is_private

assert True == ipaddress.ip_network('2001:db8::/32').is_private

assert True == ipaddress.ip_network('2001:10::/28').is_private

assert True == ipaddress.ip_network('fc00::/7').is_private

assert True == ipaddress.ip_network('fe80::/10').is_private
print("IpaddrUnitTest::testPrivateNetworks: ok")
