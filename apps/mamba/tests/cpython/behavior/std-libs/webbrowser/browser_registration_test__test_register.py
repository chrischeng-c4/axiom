# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "behavior"
# case = "browser_registration_test__test_register"
# subject = "cpython.test_webbrowser.BrowserRegistrationTest.test_register"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_webbrowser.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_webbrowser.py::BrowserRegistrationTest::test_register
"""Auto-ported test: BrowserRegistrationTest::test_register (CPython 3.12 oracle)."""


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
def _check_registration(preferred):

    class ExampleBrowser:
        pass
    expected_tryorder = []
    expected_browsers = {}

    assert webbrowser._tryorder == expected_tryorder

    assert webbrowser._browsers == expected_browsers
    webbrowser.register('Example1', ExampleBrowser)
    expected_tryorder = ['Example1']
    expected_browsers['example1'] = [ExampleBrowser, None]

    assert webbrowser._tryorder == expected_tryorder

    assert webbrowser._browsers == expected_browsers
    instance = ExampleBrowser()
    if preferred is not None:
        webbrowser.register('example2', ExampleBrowser, instance, preferred=preferred)
    else:
        webbrowser.register('example2', ExampleBrowser, instance)
    if preferred:
        expected_tryorder = ['example2', 'Example1']
    else:
        expected_tryorder = ['Example1', 'example2']
    expected_browsers['example2'] = [ExampleBrowser, instance]

    assert webbrowser._tryorder == expected_tryorder

    assert webbrowser._browsers == expected_browsers
self__saved_tryorder = webbrowser._tryorder
webbrowser._tryorder = []
self__saved_browsers = webbrowser._browsers
webbrowser._browsers = {}
_check_registration(preferred=False)
print("BrowserRegistrationTest::test_register: ok")
