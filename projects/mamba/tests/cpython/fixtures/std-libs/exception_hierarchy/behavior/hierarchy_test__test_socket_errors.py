# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_hierarchy"
# dimension = "behavior"
# case = "hierarchy_test__test_socket_errors"
# subject = "cpython.test_exception_hierarchy.HierarchyTest.test_socket_errors"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exception_hierarchy.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exception_hierarchy.py::HierarchyTest::test_socket_errors
"""Auto-ported test: HierarchyTest::test_socket_errors (CPython 3.12 oracle)."""


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

assert socket.error is OSError

assert socket.gaierror.__base__ is OSError

assert socket.herror.__base__ is OSError

assert socket.timeout is TimeoutError
print("HierarchyTest::test_socket_errors: ok")
