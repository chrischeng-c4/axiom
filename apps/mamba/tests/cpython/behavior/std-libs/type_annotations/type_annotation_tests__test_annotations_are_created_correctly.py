# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_annotations"
# dimension = "behavior"
# case = "type_annotation_tests__test_annotations_are_created_correctly"
# subject = "cpython.test_type_annotations.TypeAnnotationTests.test_annotations_are_created_correctly"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_annotations.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_annotations.py::TypeAnnotationTests::test_annotations_are_created_correctly
"""Auto-ported test: TypeAnnotationTests::test_annotations_are_created_correctly (CPython 3.12 oracle)."""


import textwrap
import unittest
from test.support import run_code


# --- test body ---
class C:
    a: int = 3
    b: str = 4

assert '__annotations__' in C.__dict__
del C.__annotations__

assert not '__annotations__' in C.__dict__
print("TypeAnnotationTests::test_annotations_are_created_correctly: ok")
