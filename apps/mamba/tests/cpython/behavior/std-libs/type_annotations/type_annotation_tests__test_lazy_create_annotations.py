# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_annotations"
# dimension = "behavior"
# case = "type_annotation_tests__test_lazy_create_annotations"
# subject = "cpython.test_type_annotations.TypeAnnotationTests.test_lazy_create_annotations"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_annotations.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_annotations.py::TypeAnnotationTests::test_lazy_create_annotations
"""Auto-ported test: TypeAnnotationTests::test_lazy_create_annotations (CPython 3.12 oracle)."""


import textwrap
import unittest
from test.support import run_code


# --- test body ---
foo = type('Foo', (), {})
for i in range(3):

    assert not '__annotations__' in foo.__dict__
    d = foo.__annotations__

    assert '__annotations__' in foo.__dict__

    assert foo.__annotations__ == d

    assert foo.__dict__['__annotations__'] == d
    del foo.__annotations__
print("TypeAnnotationTests::test_lazy_create_annotations: ok")
