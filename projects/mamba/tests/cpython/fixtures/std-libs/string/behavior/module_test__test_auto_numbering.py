# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "module_test__test_auto_numbering"
# subject = "cpython.test_string.ModuleTest.test_auto_numbering"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_string.py::ModuleTest::test_auto_numbering
"""Auto-ported test: ModuleTest::test_auto_numbering (CPython 3.12 oracle)."""


import unittest
import string
from string import Template


class Bag:
    pass

class Mapping:

    def __getitem__(self, name):
        obj = self
        for part in name.split('.'):
            try:
                obj = getattr(obj, part)
            except AttributeError:
                raise KeyError(name)
        return obj


# --- test body ---
fmt = string.Formatter()

assert fmt.format('foo{}{}', 'bar', 6) == 'foo{}{}'.format('bar', 6)

assert fmt.format('foo{1}{num}{1}', None, 'bar', num=6) == 'foo{1}{num}{1}'.format(None, 'bar', num=6)

assert fmt.format('{:^{}}', 'bar', 6) == '{:^{}}'.format('bar', 6)

assert fmt.format('{:^{}} {}', 'bar', 6, 'X') == '{:^{}} {}'.format('bar', 6, 'X')

assert fmt.format('{:^{pad}}{}', 'foo', 'bar', pad=6) == '{:^{pad}}{}'.format('foo', 'bar', pad=6)
try:
    fmt.format('foo{1}{}', 'bar', 6)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    fmt.format('foo{}{1}', 'bar', 6)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("ModuleTest::test_auto_numbering: ok")
