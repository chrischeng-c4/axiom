# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "test_template__test_pattern_override"
# subject = "cpython.test_string.TestTemplate.test_pattern_override"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_string.py::TestTemplate::test_pattern_override
"""Auto-ported test: TestTemplate::test_pattern_override (CPython 3.12 oracle)."""


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
    pattern = '\n            (?P<escaped>@{2})                   |\n            @(?P<named>[_a-z][._a-z0-9]*)       |\n            @{(?P<braced>[_a-z][._a-z0-9]*)}    |\n            (?P<invalid>@)\n            '
m = Mapping()
m.bag = Bag()
m.bag.foo = Bag()
m.bag.foo.who = 'tim'
m.bag.what = 'ham'
s = MyPattern('@bag.foo.who likes to eat a bag of @bag.what')

assert s.substitute(m) == 'tim likes to eat a bag of ham'

class BadPattern(Template):
    pattern = '\n            (?P<badname>.*)                     |\n            (?P<escaped>@{2})                   |\n            @(?P<named>[_a-z][._a-z0-9]*)       |\n            @{(?P<braced>[_a-z][._a-z0-9]*)}    |\n            (?P<invalid>@)                      |\n            '
s = BadPattern('@bag.foo.who likes to eat a bag of @bag.what')

try:
    s.substitute({})
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    s.safe_substitute({})
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("TestTemplate::test_pattern_override: ok")
