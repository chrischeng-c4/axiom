# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "resource"
# dimension = "behavior"
# case = "resource_test__test_fsize_toobig"
# subject = "cpython.test_resource.ResourceTest.test_fsize_toobig"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_resource.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_resource.py::ResourceTest::test_fsize_toobig
"""Auto-ported test: ResourceTest::test_fsize_toobig (CPython 3.12 oracle)."""


import contextlib
import sys
import unittest
from test import support
from test.support import import_helper
from test.support import os_helper
import time


resource = import_helper.import_module('resource')


# --- test body ---
too_big = 10 ** 50
try:
    cur, max = resource.getrlimit(resource.RLIMIT_FSIZE)
except AttributeError:
    pass
else:
    try:
        resource.setrlimit(resource.RLIMIT_FSIZE, (too_big, max))
    except (OverflowError, ValueError):
        pass
    try:
        resource.setrlimit(resource.RLIMIT_FSIZE, (max, too_big))
    except (OverflowError, ValueError):
        pass
print("ResourceTest::test_fsize_toobig: ok")
