# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exception_group"
# dimension = "behavior"
# case = "deep_recursion_in_split_and_subgroup__test_deep_subgroup"
# subject = "cpython.test_exception_group.DeepRecursionInSplitAndSubgroup.test_deep_subgroup"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exception_group.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exception_group.py::DeepRecursionInSplitAndSubgroup::test_deep_subgroup
"""Auto-ported test: DeepRecursionInSplitAndSubgroup::test_deep_subgroup (CPython 3.12 oracle)."""


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
def make_deep_eg():
    e = TypeError(1)
    for i in range(C_RECURSION_LIMIT + 1):
        e = ExceptionGroup('eg', [e])
    return e
e = make_deep_eg()
try:
    e.subgroup(TypeError)
    raise AssertionError('expected RecursionError')
except RecursionError:
    pass
print("DeepRecursionInSplitAndSubgroup::test_deep_subgroup: ok")
