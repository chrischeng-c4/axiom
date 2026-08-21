# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "test_template__test_idpattern_override"
# subject = "cpython.test_string.TestTemplate.test_idpattern_override"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::TestTemplate::test_idpattern_override
"""Auto-ported test: TestTemplate::test_idpattern_override (CPython 3.12 oracle)."""


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
class PathPattern(Template):
    idpattern = '[_a-z][._a-z0-9]*'
m = Mapping()
m.bag = Bag()
m.bag.foo = Bag()
m.bag.foo.who = 'tim'
m.bag.what = 'ham'
s = PathPattern('$bag.foo.who likes to eat a bag of $bag.what')

assert s.substitute(m) == 'tim likes to eat a bag of ham'
print("TestTemplate::test_idpattern_override: ok")
