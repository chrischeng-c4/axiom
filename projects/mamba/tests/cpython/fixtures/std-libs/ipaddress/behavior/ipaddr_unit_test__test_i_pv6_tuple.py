# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipaddr_unit_test__test_i_pv6_tuple"
# subject = "cpython.test_ipaddress.IpaddrUnitTest.testIPv6Tuple"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ipaddress.py::IpaddrUnitTest::testIPv6Tuple
"""Auto-ported test: IpaddrUnitTest::testIPv6Tuple (CPython 3.12 oracle)."""


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
ip = ipaddress.IPv6Address('2001:db8::')
net = ipaddress.IPv6Network('2001:db8::/128')

assert ipaddress.IPv6Network(('2001:db8::', '128')) == net

assert ipaddress.IPv6Network((42540766411282592856903984951653826560, 128)) == net

assert ipaddress.IPv6Network((ip, '128')) == net
ip = ipaddress.IPv6Address('2001:db8::')
net = ipaddress.IPv6Network('2001:db8::/96')

assert ipaddress.IPv6Network(('2001:db8::', '96')) == net

assert ipaddress.IPv6Network((42540766411282592856903984951653826560, 96)) == net

assert ipaddress.IPv6Network((ip, '96')) == net
ip_scoped = ipaddress.IPv6Address('2001:db8::%scope')
ip = ipaddress.IPv6Address('2001:db8::1')
try:
    ipaddress.IPv6Network(('2001:db8::1', 96))
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    ipaddress.IPv6Network((42540766411282592856903984951653826561, 96))
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    ipaddress.IPv6Network((ip, 96))
    raise AssertionError('expected ValueError')
except ValueError:
    pass
net = ipaddress.IPv6Network('2001:db8::/96')

assert ipaddress.IPv6Network(('2001:db8::1', 96), strict=False) == net

assert ipaddress.IPv6Network((42540766411282592856903984951653826561, 96), strict=False) == net

assert ipaddress.IPv6Network((ip, 96), strict=False) == net

assert ipaddress.IPv6Interface(('2001:db8::1', '96')) == ipaddress.IPv6Interface('2001:db8::1/96')

assert ipaddress.IPv6Interface((42540766411282592856903984951653826561, '96')) == ipaddress.IPv6Interface('2001:db8::1/96')
ip_scoped = ipaddress.IPv6Address('2001:db8::1%scope')
try:
    ipaddress.IPv6Network(('2001:db8::1%scope', 96))
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    ipaddress.IPv6Network((ip_scoped, 96))
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    ipaddress.IPv6Network(('2001:db8::1', '255.255.255.0'))
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    ipaddress.ip_network(('2001:db8::1', '255.255.255.0'))
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("IpaddrUnitTest::testIPv6Tuple: ok")
