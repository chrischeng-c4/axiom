# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_literal_constructor"
# subject = "cpython.test_dict.DictTest.test_literal_constructor"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_literal_constructor
"""Auto-ported test: DictTest::test_literal_constructor (CPython 3.12 oracle)."""


import collections
import collections.abc
import gc
import pickle
import random
import string
import sys
import unittest
import weakref
from test import support
from test.support import import_helper, C_RECURSION_LIMIT
from test import mapping_tests


class GeneralMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = dict

class Dict(dict):
    pass

class SubclassMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = Dict


# --- test body ---
for n in (0, 1, 6, 256, 400):
    items = [(''.join(random.sample(string.ascii_letters, 8)), i) for i in range(n)]
    random.shuffle(items)
    formatted_items = ('{!r}: {:d}'.format(k, v) for k, v in items)
    dictliteral = '{' + ', '.join(formatted_items) + '}'

    assert eval(dictliteral) == dict(items)
print("DictTest::test_literal_constructor: ok")
