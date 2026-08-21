# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "test_uuid_without_ext_module__test_uuid1_time"
# subject = "cpython.test_uuid.TestUUIDWithoutExtModule.test_uuid1_time"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_uuid.py::TestUUIDWithoutExtModule::test_uuid1_time
"""Auto-ported test: TestUUIDWithoutExtModule::test_uuid1_time (CPython 3.12 oracle)."""


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
uuid = None
uuid = py_uuid
with mock.patch.object(uuid, '_has_uuid_generate_time_safe', False), mock.patch.object(uuid, '_generate_time_safe', None), mock.patch.object(uuid, '_last_timestamp', None), mock.patch.object(uuid, 'getnode', return_value=93328246233727), mock.patch('time.time_ns', return_value=1545052026752910643), mock.patch('random.getrandbits', return_value=5317):
    u = uuid.uuid1()

    assert u == uuid.UUID('a7a55b92-01fc-11e9-94c5-54e1acf6da7f')
with mock.patch.object(uuid, '_has_uuid_generate_time_safe', False), mock.patch.object(uuid, '_generate_time_safe', None), mock.patch.object(uuid, '_last_timestamp', None), mock.patch('time.time_ns', return_value=1545052026752910643):
    u = uuid.uuid1(node=93328246233727, clock_seq=5317)

    assert u == uuid.UUID('a7a55b92-01fc-11e9-94c5-54e1acf6da7f')
print("TestUUIDWithoutExtModule::test_uuid1_time: ok")
