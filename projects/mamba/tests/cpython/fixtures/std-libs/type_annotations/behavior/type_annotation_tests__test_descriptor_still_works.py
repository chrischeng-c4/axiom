# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_annotations"
# dimension = "behavior"
# case = "type_annotation_tests__test_descriptor_still_works"
# subject = "cpython.test_type_annotations.TypeAnnotationTests.test_descriptor_still_works"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_annotations.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_annotations.py::TypeAnnotationTests::test_descriptor_still_works
"""Auto-ported test: TypeAnnotationTests::test_descriptor_still_works (CPython 3.12 oracle)."""


import textwrap
import unittest
from test.support import run_code


# --- test body ---
class C:

    def __init__(self, name=None, bases=None, d=None):
        self.my_annotations = None

    @property
    def __annotations__(self):
        if not hasattr(self, 'my_annotations'):
            self.my_annotations = {}
        if not isinstance(self.my_annotations, dict):
            self.my_annotations = {}
        return self.my_annotations

    @__annotations__.setter
    def __annotations__(self, value):
        if not isinstance(value, dict):
            raise ValueError('can only set __annotations__ to a dict')
        self.my_annotations = value

    @__annotations__.deleter
    def __annotations__(self):
        if getattr(self, 'my_annotations', False) is None:
            raise AttributeError('__annotations__')
        self.my_annotations = None
c = C()

assert c.__annotations__ == {}
d = {'a': 'int'}
c.__annotations__ = d

assert c.__annotations__ == d
try:
    c.__annotations__ = 123
    raise AssertionError('expected ValueError')
except ValueError:
    pass
del c.__annotations__
try:
    del c.__annotations__
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass

assert c.__annotations__ == {}

class D(metaclass=C):
    pass

assert D.__annotations__ == {}
d = {'a': 'int'}
D.__annotations__ = d

assert D.__annotations__ == d
try:
    D.__annotations__ = 123
    raise AssertionError('expected ValueError')
except ValueError:
    pass
del D.__annotations__
try:
    del D.__annotations__
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass

assert D.__annotations__ == {}
print("TypeAnnotationTests::test_descriptor_still_works: ok")
