# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_hierarchy"
# dimension = "behavior"
# case = "hierarchy_test__test_try_except"
# subject = "cpython.test_exception_hierarchy.HierarchyTest.test_try_except"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exception_hierarchy.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exception_hierarchy.py::HierarchyTest::test_try_except
"""Auto-ported test: HierarchyTest::test_try_except (CPython 3.12 oracle)."""


import builtins
import os
import select
import socket
import unittest
import errno
from errno import EEXIST


class SubOSError(OSError):
    pass

class SubOSErrorWithInit(OSError):

    def __init__(self, message, bar):
        self.bar = bar
        super().__init__(message)

class SubOSErrorWithNew(OSError):

    def __new__(cls, message, baz):
        self = super().__new__(cls, message)
        self.baz = baz
        return self

class SubOSErrorCombinedInitFirst(SubOSErrorWithInit, SubOSErrorWithNew):
    pass

class SubOSErrorCombinedNewFirst(SubOSErrorWithNew, SubOSErrorWithInit):
    pass

class SubOSErrorWithStandaloneInit(OSError):

    def __init__(self):
        pass


# --- test body ---
filename = 'some_hopefully_non_existing_file'
try:
    open(filename)
except FileNotFoundError:
    pass
else:

    raise AssertionError('should have raised a FileNotFoundError')

assert not os.path.exists(filename)
try:
    os.unlink(filename)
except FileNotFoundError:
    pass
else:

    raise AssertionError('should have raised a FileNotFoundError')
print("HierarchyTest::test_try_except: ok")
