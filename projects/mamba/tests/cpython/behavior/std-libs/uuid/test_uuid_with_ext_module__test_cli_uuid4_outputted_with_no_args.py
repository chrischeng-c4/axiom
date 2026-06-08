# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "test_uuid_with_ext_module__test_cli_uuid4_outputted_with_no_args"
# subject = "cpython.test_uuid.TestUUIDWithExtModule.test_cli_uuid4_outputted_with_no_args"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_uuid.py::TestUUIDWithExtModule::test_cli_uuid4_outputted_with_no_args
"""Auto-ported test: TestUUIDWithExtModule::test_cli_uuid4_outputted_with_no_args (CPython 3.12 oracle)."""


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
stdout = io.StringIO()
with contextlib.redirect_stdout(stdout):
    uuid.main()
output = stdout.getvalue().strip()
uuid_output = uuid.UUID(output)

assert output == str(uuid_output)

assert uuid_output.version == 4
print("TestUUIDWithExtModule::test_cli_uuid4_outputted_with_no_args: ok")
