# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "behavior"
# case = "import_test__test_get"
# subject = "cpython.test_webbrowser.ImportTest.test_get"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_webbrowser.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_webbrowser.py::ImportTest::test_get
"""Auto-ported test: ImportTest::test_get (CPython 3.12 oracle)."""


import webbrowser
import unittest
import os
import sys
import subprocess
from unittest import mock
from test import support
from test.support import import_helper
from test.support import os_helper


if not support.has_subprocess_support:
    raise unittest.SkipTest('test webserver requires subprocess')

URL = 'https://www.example.com'

CMD_NAME = 'test'

class PopenMock(mock.MagicMock):

    def poll(self):
        return 0

    def wait(self, seconds=None):
        return 0

class CommandTestMixin:

    def _test(self, meth, *, args=[URL], kw={}, options, arguments):
        """Given a web browser instance method name along with arguments and
        keywords for same (which defaults to the single argument URL), creates
        a browser instance from the class pointed to by self.browser, calls the
        indicated instance method with the indicated arguments, and compares
        the resulting options and arguments passed to Popen by the browser
        instance against the 'options' and 'args' lists.  Options are compared
        in a position independent fashion, and the arguments are compared in
        sequence order to whatever is left over after removing the options.

        """
        popen = PopenMock()
        support.patch(self, subprocess, 'Popen', popen)
        browser = self.browser_class(name=CMD_NAME)
        getattr(browser, meth)(*args, **kw)
        popen_args = subprocess.Popen.call_args[0][0]
        self.assertEqual(popen_args[0], CMD_NAME)
        popen_args.pop(0)
        for option in options:
            self.assertIn(option, popen_args)
            popen_args.pop(popen_args.index(option))
        self.assertEqual(popen_args, arguments)


# --- test body ---
webbrowser = import_helper.import_fresh_module('webbrowser')

assert webbrowser._tryorder is None

assert not webbrowser._browsers
try:
    webbrowser.get('fakebrowser')
    raise AssertionError('expected webbrowser.Error')
except webbrowser.Error:
    pass

assert webbrowser._tryorder is not None
print("ImportTest::test_get: ok")
