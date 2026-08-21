# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exception_group"
# dimension = "behavior"
# case = "nested_exception_group_basics_test__test_nested_group_matches_template"
# subject = "cpython.test_exception_group.NestedExceptionGroupBasicsTest.test_nested_group_matches_template"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exception_group.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exception_group.py::NestedExceptionGroupBasicsTest::test_nested_group_matches_template
"""Auto-ported test: NestedExceptionGroupBasicsTest::test_nested_group_matches_template (CPython 3.12 oracle)."""


import collections.abc
import types
import unittest
from test.support import C_RECURSION_LIMIT


def create_simple_eg():
    excs = []
    try:
        try:
            raise MemoryError('context and cause for ValueError(1)')
        except MemoryError as e:
            raise ValueError(1) from e
    except ValueError as e:
        excs.append(e)
    try:
        try:
            raise OSError('context for TypeError')
        except OSError as e:
            raise TypeError(int)
    except TypeError as e:
        excs.append(e)
    try:
        try:
            raise ImportError('context for ValueError(2)')
        except ImportError as e:
            raise ValueError(2)
    except ValueError as e:
        excs.append(e)
    try:
        raise ExceptionGroup('simple eg', excs)
    except ExceptionGroup as e:
        return e

def leaf_generator(exc, tbs=None):
    if tbs is None:
        tbs = []
    tbs.append(exc.__traceback__)
    if isinstance(exc, BaseExceptionGroup):
        for e in exc.exceptions:
            yield from leaf_generator(e, tbs)
    else:
        yield (exc, tbs)
    tbs.pop()

def create_nested_eg():
    excs = []
    try:
        try:
            raise TypeError(bytes)
        except TypeError as e:
            raise ExceptionGroup('nested', [e])
    except ExceptionGroup as e:
        excs.append(e)
    try:
        try:
            raise MemoryError('out of memory')
        except MemoryError as e:
            raise ValueError(1) from e
    except ValueError as e:
        excs.append(e)
    try:
        raise ExceptionGroup('root', excs)
    except ExceptionGroup as eg:
        return eg


# --- test body ---
def assertMatchesTemplate(exc, exc_type, template):
    """ Assert that the exception matches the template

            A template describes the shape of exc. If exc is a
            leaf exception (i.e., not an exception group) then
            template is an exception instance that has the
            expected type and args value of exc. If exc is an
            exception group, then template is a list of the
            templates of its nested exceptions.
        """
    if exc_type is not None:

        assert type(exc) is exc_type
    if isinstance(exc, BaseExceptionGroup):

        assert isinstance(template, collections.abc.Sequence)

        assert len(exc.exceptions) == len(template)
        for e, t in zip(exc.exceptions, template):
            assertMatchesTemplate(e, None, t)
    else:

        assert isinstance(template, BaseException)

        assert type(exc) == type(template)

        assert exc.args == template.args
eg = create_nested_eg()
assertMatchesTemplate(eg, ExceptionGroup, [[TypeError(bytes)], ValueError(1)])
print("NestedExceptionGroupBasicsTest::test_nested_group_matches_template: ok")
