# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "test_uuid_without_ext_module__test_uuid1_eui64"
# subject = "cpython.test_uuid.TestUUIDWithoutExtModule.test_uuid1_eui64"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_uuid.py::TestUUIDWithoutExtModule::test_uuid1_eui64
"""Auto-ported test: TestUUIDWithoutExtModule::test_uuid1_eui64 (CPython 3.12 oracle)."""


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
too_large_getter = lambda: 1 << 48
with mock.patch.multiple(uuid, _node=None, _GETTERS=[too_large_getter]):
    node = uuid.getnode()

assert 0 < node < 1 << 48
try:
    uuid.uuid1(node=node)
except ValueError:

    raise AssertionError('uuid1 was given an invalid node ID')
print("TestUUIDWithoutExtModule::test_uuid1_eui64: ok")
