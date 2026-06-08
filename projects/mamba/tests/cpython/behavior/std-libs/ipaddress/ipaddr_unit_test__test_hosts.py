# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipaddr_unit_test__test_hosts"
# subject = "cpython.test_ipaddress.IpaddrUnitTest.testHosts"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ipaddress.py::IpaddrUnitTest::testHosts
"""Auto-ported test: IpaddrUnitTest::testHosts (CPython 3.12 oracle)."""


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
hosts = list(self_ipv4_network.hosts())

assert 254 == len(hosts)

assert ipaddress.IPv4Address('1.2.3.1') == hosts[0]

assert ipaddress.IPv4Address('1.2.3.254') == hosts[-1]
ipv6_network = ipaddress.IPv6Network('2001:658:22a:cafe::/120')
hosts = list(ipv6_network.hosts())

assert 255 == len(hosts)

assert ipaddress.IPv6Address('2001:658:22a:cafe::1') == hosts[0]

assert ipaddress.IPv6Address('2001:658:22a:cafe::ff') == hosts[-1]
ipv6_scoped_network = ipaddress.IPv6Network('2001:658:22a:cafe::%scope/120')
hosts = list(ipv6_scoped_network.hosts())

assert 255 == len(hosts)

assert ipaddress.IPv6Address('2001:658:22a:cafe::1') == hosts[0]

assert ipaddress.IPv6Address('2001:658:22a:cafe::ff') == hosts[-1]
addrs = [ipaddress.IPv4Address('2.0.0.0'), ipaddress.IPv4Address('2.0.0.1')]
str_args = '2.0.0.0/31'
tpl_args = ('2.0.0.0', 31)

assert addrs == list(ipaddress.ip_network(str_args).hosts())

assert addrs == list(ipaddress.ip_network(tpl_args).hosts())

assert list(ipaddress.ip_network(str_args).hosts()) == list(ipaddress.ip_network(tpl_args).hosts())
addrs = [ipaddress.IPv4Address('1.2.3.4')]
str_args = '1.2.3.4/32'
tpl_args = ('1.2.3.4', 32)

assert addrs == list(ipaddress.ip_network(str_args).hosts())

assert addrs == list(ipaddress.ip_network(tpl_args).hosts())

assert list(ipaddress.ip_network(str_args).hosts()) == list(ipaddress.ip_network(tpl_args).hosts())
addrs = [ipaddress.IPv6Address('2001:658:22a:cafe::'), ipaddress.IPv6Address('2001:658:22a:cafe::1')]
str_args = '2001:658:22a:cafe::/127'
tpl_args = ('2001:658:22a:cafe::', 127)

assert addrs == list(ipaddress.ip_network(str_args).hosts())

assert addrs == list(ipaddress.ip_network(tpl_args).hosts())

assert list(ipaddress.ip_network(str_args).hosts()) == list(ipaddress.ip_network(tpl_args).hosts())
addrs = [ipaddress.IPv6Address('2001:658:22a:cafe::1')]
str_args = '2001:658:22a:cafe::1/128'
tpl_args = ('2001:658:22a:cafe::1', 128)

assert addrs == list(ipaddress.ip_network(str_args).hosts())

assert addrs == list(ipaddress.ip_network(tpl_args).hosts())

assert list(ipaddress.ip_network(str_args).hosts()) == list(ipaddress.ip_network(tpl_args).hosts())
print("IpaddrUnitTest::testHosts: ok")
