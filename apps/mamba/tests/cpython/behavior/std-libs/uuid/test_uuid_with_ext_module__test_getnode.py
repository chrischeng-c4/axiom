# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "test_uuid_with_ext_module__test_getnode"
# subject = "cpython.test_uuid.TestUUIDWithExtModule.test_getnode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_uuid.py::TestUUIDWithExtModule::test_getnode
"""Auto-ported test: TestUUIDWithExtModule::test_getnode (CPython 3.12 oracle)."""


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
uuid = c_uuid
node1 = uuid.getnode()

assert 0 < node1 < 1 << 48
node2 = uuid.getnode()

assert node1 == node2
print("TestUUIDWithExtModule::test_getnode: ok")
