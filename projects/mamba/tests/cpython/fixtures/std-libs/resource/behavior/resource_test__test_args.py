# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "resource"
# dimension = "behavior"
# case = "resource_test__test_args"
# subject = "cpython.test_resource.ResourceTest.test_args"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_resource.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_resource.py::ResourceTest::test_args
"""Auto-ported test: ResourceTest::test_args (CPython 3.12 oracle)."""


import contextlib
import sys
import unittest
from test import support
from test.support import import_helper
from test.support import os_helper
import time


resource = import_helper.import_module('resource')


# --- test body ---

try:
    resource.getrlimit()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    resource.getrlimit(42, 42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    resource.setrlimit()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    resource.setrlimit(42, 42, 42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ResourceTest::test_args: ok")
