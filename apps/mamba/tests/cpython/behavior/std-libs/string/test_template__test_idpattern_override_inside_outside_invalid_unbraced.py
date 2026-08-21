# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "test_template__test_idpattern_override_inside_outside_invalid_unbraced"
# subject = "cpython.test_string.TestTemplate.test_idpattern_override_inside_outside_invalid_unbraced"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::TestTemplate::test_idpattern_override_inside_outside_invalid_unbraced
"""Auto-ported test: TestTemplate::test_idpattern_override_inside_outside_invalid_unbraced (CPython 3.12 oracle)."""


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
    idpattern = '[a-z]+'
    braceidpattern = '[A-Z]+'
    flags = 0
m = dict(foo='foo', BAR='BAR')
s = MyPattern('$FOO')

try:
    s.substitute(m)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
s = MyPattern('${bar}')

try:
    s.substitute(m)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("TestTemplate::test_idpattern_override_inside_outside_invalid_unbraced: ok")
