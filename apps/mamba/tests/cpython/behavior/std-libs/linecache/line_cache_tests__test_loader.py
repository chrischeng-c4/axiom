# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "line_cache_tests__test_loader"
# subject = "cpython.test_linecache.LineCacheTests.test_loader"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_linecache.py::LineCacheTests::test_loader
"""Auto-ported test: LineCacheTests::test_loader (CPython 3.12 oracle)."""


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
filename = 'scheme://path'
for loader in (None, object(), NoSourceLoader()):
    linecache.clearcache()
    module_globals = {'__name__': 'a.b.c', '__loader__': loader}

    assert linecache.getlines(filename, module_globals) == []
linecache.clearcache()
module_globals = {'__name__': 'a.b.c', '__loader__': FakeLoader()}

assert linecache.getlines(filename, module_globals) == ['source for a.b.c\n']
for spec in (None, object(), ModuleSpec('', FakeLoader())):
    linecache.clearcache()
    module_globals = {'__name__': 'a.b.c', '__loader__': FakeLoader(), '__spec__': spec}

    assert linecache.getlines(filename, module_globals) == ['source for a.b.c\n']
linecache.clearcache()
spec = ModuleSpec('x.y.z', FakeLoader())
module_globals = {'__name__': 'a.b.c', '__loader__': spec.loader, '__spec__': spec}

assert linecache.getlines(filename, module_globals) == ['source for x.y.z\n']
print("LineCacheTests::test_loader: ok")
