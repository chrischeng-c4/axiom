# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_annotations"
# dimension = "behavior"
# case = "type_annotation_tests__test_annotations_getset_raises"
# subject = "cpython.test_type_annotations.TypeAnnotationTests.test_annotations_getset_raises"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_annotations.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_annotations.py::TypeAnnotationTests::test_annotations_getset_raises
"""Auto-ported test: TypeAnnotationTests::test_annotations_getset_raises (CPython 3.12 oracle)."""


import textwrap
import unittest
from test.support import run_code


# --- test body ---
try:
    print(float.__annotations__)
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
try:
    float.__annotations__ = {}
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    del float.__annotations__
    raise AssertionError('expected TypeError')
except TypeError:
    pass
foo = type('Foo', (), {})
foo.__annotations__ = {}
del foo.__annotations__
try:
    del foo.__annotations__
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
print("TypeAnnotationTests::test_annotations_getset_raises: ok")
