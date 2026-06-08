# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "line_cache_tests__test_checkcache"
# subject = "cpython.test_linecache.LineCacheTests.test_checkcache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_linecache.py::LineCacheTests::test_checkcache
"""Auto-ported test: LineCacheTests::test_checkcache (CPython 3.12 oracle)."""


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
getline = linecache.getline
source_name = os_helper.TESTFN + '.py'
pass
with open(source_name, 'w', encoding='utf-8') as source:
    source.write(SOURCE_1)
getline(source_name, 1)
source_list = []
with open(source_name, encoding='utf-8') as source:
    for index, line in enumerate(source):

        assert line == getline(source_name, index + 1)
        source_list.append(line)
with open(source_name, 'w', encoding='utf-8') as source:
    source.write(SOURCE_2)
linecache.checkcache('dummy')
for index, line in enumerate(source_list):

    assert line == getline(source_name, index + 1)
linecache.checkcache(source_name)
with open(source_name, encoding='utf-8') as source:
    for index, line in enumerate(source):

        assert line == getline(source_name, index + 1)
        source_list.append(line)
print("LineCacheTests::test_checkcache: ok")
