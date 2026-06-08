# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "test_internals_without_ext_module__test_find_under_heading"
# subject = "cpython.test_uuid.TestInternalsWithoutExtModule.test_find_under_heading"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_uuid.py::TestInternalsWithoutExtModule::test_find_under_heading
"""Auto-ported test: TestInternalsWithoutExtModule::test_find_under_heading (CPython 3.12 oracle)."""


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
uuid = py_uuid
data = 'Name  Mtu   Network     Address           Ipkts Ierrs    Opkts Oerrs  Coll\nen0   1500  link#2      fe.ad.c.1.23.4   1714807956     0 711348489     0     0\n                        01:00:5e:00:00:01\nen0   1500  192.168.129 x071             1714807956     0 711348489     0     0\n                        224.0.0.1\nen0   1500  192.168.90  x071             1714807956     0 711348489     0     0\n                        224.0.0.1\n'
with mock.patch.multiple(uuid, _MAC_DELIM=b'.', _MAC_OMITS_LEADING_ZEROES=True, _get_command_stdout=mock_get_command_stdout(data)):
    mac = uuid._find_mac_under_heading(command='netstat', args='-ian', heading=b'Address')

assert mac == 280019184198404
print("TestInternalsWithoutExtModule::test_find_under_heading: ok")
