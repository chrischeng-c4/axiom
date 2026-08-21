# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "test_template__test_flags_override"
# subject = "cpython.test_string.TestTemplate.test_flags_override"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::TestTemplate::test_flags_override
"""Auto-ported test: TestTemplate::test_flags_override (CPython 3.12 oracle)."""


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
class MyPattern(Template):
    flags = 0
s = MyPattern('$wHO likes ${WHAT} for ${meal}')
d = dict(wHO='tim', WHAT='ham', meal='dinner', w='fred')

try:
    s.substitute(d)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert s.safe_substitute(d) == 'fredHO likes ${WHAT} for dinner'
print("TestTemplate::test_flags_override: ok")
