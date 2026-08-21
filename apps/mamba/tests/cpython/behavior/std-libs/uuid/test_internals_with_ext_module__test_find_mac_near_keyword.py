# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "test_internals_with_ext_module__test_find_mac_near_keyword"
# subject = "cpython.test_uuid.TestInternalsWithExtModule.test_find_mac_near_keyword"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_uuid.py::TestInternalsWithExtModule::test_find_mac_near_keyword
"""Auto-ported test: TestInternalsWithExtModule::test_find_mac_near_keyword (CPython 3.12 oracle)."""


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
data = '\nfake      Link encap:UNSPEC  hwaddr 00-00\ncscotun0  Link encap:UNSPEC  HWaddr 00-00-00-00-00-00-00-00-00-00-00-00-00-00-00-00\neth0      Link encap:Ethernet  HWaddr 12:34:56:78:90:ab\n'
with mock.patch.multiple(uuid, _MAC_DELIM=b':', _MAC_OMITS_LEADING_ZEROES=False, _get_command_stdout=mock_get_command_stdout(data)):
    mac = uuid._find_mac_near_keyword(command='ifconfig', args='', keywords=[b'hwaddr'], get_word_index=lambda x: x + 1)

assert mac == 20015998341291
print("TestInternalsWithExtModule::test_find_mac_near_keyword: ok")
