# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regrtest"
# dimension = "behavior"
# case = "test_utils__test_match_test"
# subject = "cpython.test_regrtest.TestUtils.test_match_test"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_regrtest.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_regrtest.py::TestUtils::test_match_test
"""Auto-ported test: TestUtils::test_match_test (CPython 3.12 oracle)."""


import contextlib
import dataclasses
import glob
import io
import locale
import os.path
import platform
import random
import re
import shlex
import signal
import subprocess
import sys
import sysconfig
import tempfile
import textwrap
import unittest
from xml.etree import ElementTree
from test import support
from test.support import os_helper
from test.libregrtest import cmdline
from test.libregrtest import main
from test.libregrtest import setup
from test.libregrtest import utils
from test.libregrtest.filter import get_match_tests, set_match_tests, match_test
from test.libregrtest.result import TestStats
from test.libregrtest.utils import normalize_test_name


'\nTests of regrtest.py.\n\nNote: test_regrtest cannot be run twice in parallel.\n'

if not support.has_subprocess_support:
    raise unittest.SkipTest('test module requires subprocess')

ROOT_DIR = os.path.join(os.path.dirname(__file__), '..', '..')

ROOT_DIR = os.path.abspath(os.path.normpath(ROOT_DIR))

LOG_PREFIX = '[0-9]+:[0-9]+:[0-9]+ (?:load avg: [0-9]+\\.[0-9]{2} )?'

RESULT_REGEX = ('passed', 'failed', 'skipped', 'interrupted', 'env changed', 'timed out', 'ran no tests', 'worker non-zero exit code')

RESULT_REGEX = f"(?:{'|'.join(RESULT_REGEX)})"

EXITCODE_BAD_TEST = 2

EXITCODE_ENV_CHANGED = 3

EXITCODE_NO_TESTS_RAN = 4

EXITCODE_RERUN_FAIL = 5

EXITCODE_INTERRUPTED = 130

TEST_INTERRUPTED = textwrap.dedent('\n    from signal import SIGINT, raise_signal\n    try:\n        raise_signal(SIGINT)\n    except ImportError:\n        import os\n        os.kill(os.getpid(), SIGINT)\n    ')

@dataclasses.dataclass(slots=True)
class Rerun:
    name: str
    match: str | None
    success: bool


# --- test body ---
class Test:

    def __init__(self, test_id):
        self.test_id = test_id

    def id(self):
        return self.test_id
patterns = get_match_tests()
pass
test_access = Test('test.test_os.FileTests.test_access')
test_chdir = Test('test.test_os.Win32ErrorTests.test_chdir')
test_copy = Test('test.test_shutil.TestCopy.test_copy')
with support.swap_attr(support, '_test_matchers', ()):
    set_match_tests([])

    assert match_test(test_access)

    assert match_test(test_chdir)
    set_match_tests(None)

    assert match_test(test_access)

    assert match_test(test_chdir)
    set_match_tests([(test_access.id(), True)])

    assert match_test(test_access)

    assert not match_test(test_chdir)
    set_match_tests([('test_os', True)])

    assert match_test(test_access)

    assert match_test(test_chdir)

    assert not match_test(test_copy)
    set_match_tests([('test_*', True)])

    assert match_test(test_access)

    assert match_test(test_chdir)
    set_match_tests([('filetests', True)])

    assert not match_test(test_access)
    set_match_tests([('FileTests', True)])

    assert match_test(test_access)
    set_match_tests([('*test_os.*.test_*', True)])

    assert match_test(test_access)

    assert match_test(test_chdir)

    assert not match_test(test_copy)
    set_match_tests([(test_access.id(), True), (test_chdir.id(), True)])

    assert match_test(test_access)

    assert match_test(test_chdir)

    assert not match_test(test_copy)
    set_match_tests([('test_access', True), ('DONTMATCH', True)])

    assert match_test(test_access)

    assert not match_test(test_chdir)
with support.swap_attr(support, '_test_matchers', ()):
    set_match_tests([(test_access.id(), False)])

    assert not match_test(test_access)

    assert match_test(test_chdir)
    set_match_tests([('test_os', False)])

    assert not match_test(test_access)

    assert not match_test(test_chdir)

    assert match_test(test_copy)
    set_match_tests([('test_*', False)])

    assert not match_test(test_access)

    assert not match_test(test_chdir)
    set_match_tests([('filetests', False)])

    assert match_test(test_access)
    set_match_tests([('FileTests', False)])

    assert not match_test(test_access)
    set_match_tests([('*test_os.*.test_*', False)])

    assert not match_test(test_access)

    assert not match_test(test_chdir)

    assert match_test(test_copy)
    set_match_tests([(test_access.id(), False), (test_chdir.id(), False)])

    assert not match_test(test_access)

    assert not match_test(test_chdir)

    assert match_test(test_copy)
    set_match_tests([('test_access', False), ('DONTMATCH', False)])

    assert not match_test(test_access)

    assert match_test(test_chdir)
with support.swap_attr(support, '_test_matchers', ()):
    set_match_tests([('*test_os', False), ('test_access', True)])

    assert match_test(test_access)

    assert not match_test(test_chdir)

    assert match_test(test_copy)
    set_match_tests([('*test_os', True), ('test_access', False)])

    assert not match_test(test_access)

    assert match_test(test_chdir)

    assert not match_test(test_copy)
print("TestUtils::test_match_test: ok")
