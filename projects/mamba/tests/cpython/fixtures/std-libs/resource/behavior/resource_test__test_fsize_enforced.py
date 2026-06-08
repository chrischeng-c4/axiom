# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "resource"
# dimension = "behavior"
# case = "resource_test__test_fsize_enforced"
# subject = "cpython.test_resource.ResourceTest.test_fsize_enforced"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_resource.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_resource.py::ResourceTest::test_fsize_enforced
"""Auto-ported test: ResourceTest::test_fsize_enforced (CPython 3.12 oracle)."""


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
    cur, max = resource.getrlimit(resource.RLIMIT_FSIZE)
except AttributeError:
    pass
else:
    try:
        try:
            resource.setrlimit(resource.RLIMIT_FSIZE, (1024, max))
            limit_set = True
        except ValueError:
            limit_set = False
        f = open(os_helper.TESTFN, 'wb')
        try:
            f.write(b'X' * 1024)
            try:
                f.write(b'Y')
                f.flush()
                for i in range(5):
                    time.sleep(0.1)
                    f.flush()
            except OSError:
                if not limit_set:
                    raise
            if limit_set:
                resource.setrlimit(resource.RLIMIT_FSIZE, (cur, max))
        finally:
            f.close()
    finally:
        if limit_set:
            resource.setrlimit(resource.RLIMIT_FSIZE, (cur, max))
        os_helper.unlink(os_helper.TESTFN)
print("ResourceTest::test_fsize_enforced: ok")
