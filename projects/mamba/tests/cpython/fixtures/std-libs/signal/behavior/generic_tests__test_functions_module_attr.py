# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "generic_tests__test_functions_module_attr"
# subject = "cpython.test_signal.GenericTests.test_functions_module_attr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_signal.py::GenericTests::test_functions_module_attr
"""Auto-ported test: GenericTests::test_functions_module_attr (CPython 3.12 oracle)."""


import enum
import errno
import functools
import inspect
import os
import random
import signal
import socket
import statistics
import subprocess
import sys
import threading
import time
import unittest
from test import support
from test.support import os_helper
from test.support.script_helper import assert_python_ok, spawn_python
from test.support import threading_helper


try:
    import _testcapi
except ImportError:
    _testcapi = None

def tearDownModule():
    support.reap_children()


# --- test body ---
for name in dir(signal):
    value = getattr(signal, name)
    if inspect.isroutine(value) and (not inspect.isbuiltin(value)):

        assert value.__module__ == 'signal'
print("GenericTests::test_functions_module_attr: ok")
