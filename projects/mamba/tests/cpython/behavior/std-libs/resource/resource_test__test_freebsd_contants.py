# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "resource"
# dimension = "behavior"
# case = "resource_test__test_freebsd_contants"
# subject = "cpython.test_resource.ResourceTest.test_freebsd_contants"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_resource.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_resource.py::ResourceTest::test_freebsd_contants
"""Auto-ported test: ResourceTest::test_freebsd_contants (CPython 3.12 oracle)."""


import contextlib
import sys
import unittest
from test import support
from test.support import import_helper
from test.support import os_helper
import time


resource = import_helper.import_module('resource')


# --- test body ---
for attr in ['SWAP', 'SBSIZE', 'NPTS']:
    with contextlib.suppress(AttributeError):

        assert isinstance(getattr(resource, 'RLIMIT_' + attr), int)
print("ResourceTest::test_freebsd_contants: ok")
