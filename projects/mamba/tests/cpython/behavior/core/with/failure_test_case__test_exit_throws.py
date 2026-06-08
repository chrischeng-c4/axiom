# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "with"
# dimension = "behavior"
# case = "failure_test_case__test_exit_throws"
# subject = "cpython.test_with.FailureTestCase.testExitThrows"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_with.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_with.py::FailureTestCase::testExitThrows
"""Auto-ported test: FailureTestCase::testExitThrows (CPython 3.12 oracle)."""


import sys
import traceback
import unittest
from collections import deque
from contextlib import _GeneratorContextManager, contextmanager, nullcontext


'Unit tests for the with statement specified in PEP 343.'

__author__ = 'Mike Bland'

__email__ = 'mbland at acm dot org'

class MockContextManager(_GeneratorContextManager):

    def __init__(self, *args):
        super().__init__(*args)
        self.enter_called = False
        self.exit_called = False
        self.exit_args = None

    def __enter__(self):
        self.enter_called = True
        return _GeneratorContextManager.__enter__(self)

    def __exit__(self, type, value, traceback):
        self.exit_called = True
        self.exit_args = (type, value, traceback)
        return _GeneratorContextManager.__exit__(self, type, value, traceback)

def mock_contextmanager(func):

    def helper(*args, **kwds):
        return MockContextManager(func, args, kwds)
    return helper

class MockResource(object):

    def __init__(self):
        self.yielded = False
        self.stopped = False

@mock_contextmanager
def mock_contextmanager_generator():
    mock = MockResource()
    try:
        mock.yielded = True
        yield mock
    finally:
        mock.stopped = True

class Nested(object):

    def __init__(self, *managers):
        self.managers = managers
        self.entered = None

    def __enter__(self):
        if self.entered is not None:
            raise RuntimeError('Context is not reentrant')
        self.entered = deque()
        vars = []
        try:
            for mgr in self.managers:
                vars.append(mgr.__enter__())
                self.entered.appendleft(mgr)
        except:
            if not self.__exit__(*sys.exc_info()):
                raise
        return vars

    def __exit__(self, *exc_info):
        ex = exc_info
        for mgr in self.entered:
            try:
                if mgr.__exit__(*ex):
                    ex = (None, None, None)
            except BaseException as e:
                ex = (type(e), e, e.__traceback__)
        self.entered = None
        if ex is not exc_info:
            raise ex

class MockNested(Nested):

    def __init__(self, *managers):
        Nested.__init__(self, *managers)
        self.enter_called = False
        self.exit_called = False
        self.exit_args = None

    def __enter__(self):
        self.enter_called = True
        return Nested.__enter__(self)

    def __exit__(self, *exc_info):
        self.exit_called = True
        self.exit_args = exc_info
        return Nested.__exit__(self, *exc_info)

class ContextmanagerAssertionMixin(object):

    def setUp(self):
        self.TEST_EXCEPTION = RuntimeError('test exception')

    def assertInWithManagerInvariants(self, mock_manager):
        self.assertTrue(mock_manager.enter_called)
        self.assertFalse(mock_manager.exit_called)
        self.assertEqual(mock_manager.exit_args, None)

    def assertAfterWithManagerInvariants(self, mock_manager, exit_args):
        self.assertTrue(mock_manager.enter_called)
        self.assertTrue(mock_manager.exit_called)
        self.assertEqual(mock_manager.exit_args, exit_args)

    def assertAfterWithManagerInvariantsNoError(self, mock_manager):
        self.assertAfterWithManagerInvariants(mock_manager, (None, None, None))

    def assertInWithGeneratorInvariants(self, mock_generator):
        self.assertTrue(mock_generator.yielded)
        self.assertFalse(mock_generator.stopped)

    def assertAfterWithGeneratorInvariantsNoError(self, mock_generator):
        self.assertTrue(mock_generator.yielded)
        self.assertTrue(mock_generator.stopped)

    def raiseTestException(self):
        raise self.TEST_EXCEPTION

    def assertAfterWithManagerInvariantsWithError(self, mock_manager, exc_type=None):
        self.assertTrue(mock_manager.enter_called)
        self.assertTrue(mock_manager.exit_called)
        if exc_type is None:
            self.assertEqual(mock_manager.exit_args[1], self.TEST_EXCEPTION)
            exc_type = type(self.TEST_EXCEPTION)
        self.assertEqual(mock_manager.exit_args[0], exc_type)
        self.assertIsInstance(mock_manager.exit_args[1], exc_type)
        self.assertIsNot(mock_manager.exit_args[2], None)

    def assertAfterWithGeneratorInvariantsWithError(self, mock_generator):
        self.assertTrue(mock_generator.yielded)
        self.assertTrue(mock_generator.stopped)


# --- test body ---
def assertRaisesSyntaxError(codestr):

    def shouldRaiseSyntaxError(s):
        compile(s, '', 'single')

    try:
        shouldRaiseSyntaxError(codestr)
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass

class ExitThrows(object):

    def __enter__(self):
        return

    def __exit__(self, *args):
        raise RuntimeError(42)

def shouldThrow():
    with ExitThrows():
        pass

try:
    shouldThrow()
    raise AssertionError('expected RuntimeError')
except RuntimeError:
    pass
print("FailureTestCase::testExitThrows: ok")
