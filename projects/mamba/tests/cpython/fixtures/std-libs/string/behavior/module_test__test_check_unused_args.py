# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "module_test__test_check_unused_args"
# subject = "cpython.test_string.ModuleTest.test_check_unused_args"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_string.py::ModuleTest::test_check_unused_args
"""Auto-ported test: ModuleTest::test_check_unused_args (CPython 3.12 oracle)."""


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
class CheckAllUsedFormatter(string.Formatter):

    def check_unused_args(self, used_args, args, kwargs):
        unused_args = set(kwargs.keys())
        unused_args.update(range(0, len(args)))
        for arg in used_args:
            unused_args.remove(arg)
        if unused_args:
            raise ValueError('unused arguments')
fmt = CheckAllUsedFormatter()

assert fmt.format('{0}', 10) == '10'

assert fmt.format('{0}{i}', 10, i=100) == '10100'

assert fmt.format('{0}{i}{1}', 10, 20, i=100) == '1010020'

try:
    fmt.format('{0}{i}{1}', 10, 20, i=100, j=0)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    fmt.format('{0}', 10, 20)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    fmt.format('{0}', 10, 20, i=100)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    fmt.format('{i}', 10, 20, i=100)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("ModuleTest::test_check_unused_args: ok")
