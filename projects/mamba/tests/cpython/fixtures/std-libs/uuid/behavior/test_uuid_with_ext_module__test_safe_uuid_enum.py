# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "test_uuid_with_ext_module__test_safe_uuid_enum"
# subject = "cpython.test_uuid.TestUUIDWithExtModule.test_safe_uuid_enum"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_uuid.py::TestUUIDWithExtModule::test_safe_uuid_enum
"""Auto-ported test: TestUUIDWithExtModule::test_safe_uuid_enum (CPython 3.12 oracle)."""


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

class CheckedSafeUUID(enum.Enum):
    safe = 0
    unsafe = -1
    unknown = None
enum._test_simple_enum(CheckedSafeUUID, py_uuid.SafeUUID)
print("TestUUIDWithExtModule::test_safe_uuid_enum: ok")
