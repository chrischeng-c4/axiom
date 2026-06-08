# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "test_internals_with_ext_module__test_find_under_heading_ipv6"
# subject = "cpython.test_uuid.TestInternalsWithExtModule.test_find_under_heading_ipv6"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_uuid.py::TestInternalsWithExtModule::test_find_under_heading_ipv6
"""Auto-ported test: TestInternalsWithExtModule::test_find_under_heading_ipv6 (CPython 3.12 oracle)."""


import unittest
from test import support
from test.support import import_helper
import builtins
import contextlib
import copy
import enum
import io
import os
import pickle
import sys
import weakref
from unittest import mock


py_uuid = import_helper.import_fresh_module('uuid', blocked=['_uuid'])

c_uuid = import_helper.import_fresh_module('uuid', fresh=['_uuid'])

def importable(name):
    try:
        __import__(name)
        return True
    except ModuleNotFoundError:
        return False

def mock_get_command_stdout(data):

    def get_command_stdout(command, args):
        return io.BytesIO(data.encode())
    return get_command_stdout


# --- test body ---
uuid = c_uuid
data = 'Name    Mtu Network       Address              Ipkts Ierrs Idrop    Opkts Oerrs  Coll\nvtnet  1500 <Link#1>      52:54:00:9d:0e:67    10017     0     0     8174     0     0\nvtnet     - fe80::%vtnet0 fe80::5054:ff:fe9        0     -     -        4     -     -\nvtnet     - 192.168.122.0 192.168.122.45        8844     -     -     8171     -     -\nlo0   16384 <Link#2>      lo0                 260148     0     0   260148     0     0\nlo0       - ::1/128       ::1                    193     -     -      193     -     -\n                          ff01::1%lo0\n                          ff02::2:2eb7:74fa\n                          ff02::2:ff2e:b774\n                          ff02::1%lo0\n                          ff02::1:ff00:1%lo\nlo0       - fe80::%lo0/64 fe80::1%lo0              0     -     -        0     -     -\n                          ff01::1%lo0\n                          ff02::2:2eb7:74fa\n                          ff02::2:ff2e:b774\n                          ff02::1%lo0\n                          ff02::1:ff00:1%lo\nlo0       - 127.0.0.0/8   127.0.0.1           259955     -     -   259955     -     -\n                          224.0.0.1\n'
with mock.patch.multiple(uuid, _MAC_DELIM=b':', _MAC_OMITS_LEADING_ZEROES=False, _get_command_stdout=mock_get_command_stdout(data)):
    mac = uuid._find_mac_under_heading(command='netstat', args='-ian', heading=b'Address')

assert mac == 90520741023335
print("TestInternalsWithExtModule::test_find_under_heading_ipv6: ok")
