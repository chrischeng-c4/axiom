# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_hierarchy"
# dimension = "behavior"
# case = "attributes_test__test_blockingioerror"
# subject = "cpython.test_exception_hierarchy.AttributesTest.test_blockingioerror"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exception_hierarchy.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exception_hierarchy.py::AttributesTest::test_blockingioerror
"""Auto-ported test: AttributesTest::test_blockingioerror (CPython 3.12 oracle)."""


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
args = ('a', 'b', 'c', 'd', 'e')
for n in range(6):
    e = BlockingIOError(*args[:n])
    try:
        e.characters_written
        raise AssertionError('expected AttributeError')
    except AttributeError:
        pass
    try:
        del e.characters_written
        raise AssertionError('expected AttributeError')
    except AttributeError:
        pass
e = BlockingIOError('a', 'b', 3)

assert e.characters_written == 3
e.characters_written = 5

assert e.characters_written == 5
del e.characters_written
try:
    e.characters_written
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
print("AttributesTest::test_blockingioerror: ok")
