# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "line_cache_tests__test_lazycache_provide_after_failed_lookup"
# subject = "cpython.test_linecache.LineCacheTests.test_lazycache_provide_after_failed_lookup"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_linecache.py::LineCacheTests::test_lazycache_provide_after_failed_lookup
"""Auto-ported test: LineCacheTests::test_lazycache_provide_after_failed_lookup (CPython 3.12 oracle)."""


import linecache
import unittest
import os.path
import tempfile
import tokenize
from importlib.machinery import ModuleSpec
from test import support
from test.support import os_helper


' Tests for the linecache module '

FILENAME = linecache.__file__

NONEXISTENT_FILENAME = FILENAME + '.missing'

INVALID_NAME = '!@$)(!@#_1'

EMPTY = ''

TEST_PATH = os.path.dirname(__file__)

MODULES = 'linecache abc'.split()

MODULE_PATH = os.path.dirname(FILENAME)

SOURCE_1 = '\n" Docstring "\n\ndef function():\n    return result\n\n'

SOURCE_2 = '\ndef f():\n    return 1 + 1\n\na = f()\n\n'

SOURCE_3 = '\ndef f():\n    return 3'

class TempFile:

    def setUp(self):
        super().setUp()
        with tempfile.NamedTemporaryFile(delete=False) as fp:
            self.file_name = fp.name
            fp.write(self.file_byte_string)
        self.addCleanup(os_helper.unlink, self.file_name)

class FakeLoader:

    def get_source(self, fullname):
        return f'source for {fullname}'

class NoSourceLoader:

    def get_source(self, fullname):
        return None


# --- test body ---
linecache.clearcache()
lines = linecache.getlines(NONEXISTENT_FILENAME, globals())
linecache.clearcache()
linecache.getlines(NONEXISTENT_FILENAME)
linecache.lazycache(NONEXISTENT_FILENAME, globals())

assert lines == linecache.updatecache(NONEXISTENT_FILENAME)
print("LineCacheTests::test_lazycache_provide_after_failed_lookup: ok")
