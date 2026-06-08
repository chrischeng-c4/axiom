# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "reprlib"
# dimension = "behavior"
# case = "test_recursive_repr__test_assigned_attributes"
# subject = "cpython.test_reprlib.TestRecursiveRepr.test_assigned_attributes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_reprlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_reprlib.py::TestRecursiveRepr::test_assigned_attributes
"""Auto-ported test: TestRecursiveRepr::test_assigned_attributes (CPython 3.12 oracle)."""


import sys
import os
import shutil
import importlib
import importlib.util
import unittest
import textwrap
from test.support import verbose
from test.support.os_helper import create_empty_file
from reprlib import repr as r
from reprlib import Repr
from reprlib import recursive_repr


'\n  Test cases for the repr module\n  Nick Mathewson\n'

def nestedTuple(nesting):
    t = ()
    for i in range(nesting):
        t = (t,)
    return t

def write_file(path, text):
    with open(path, 'w', encoding='ASCII') as fp:
        fp.write(text)

class ClassWithRepr:

    def __init__(self, s):
        self.s = s

    def __repr__(self):
        return 'ClassWithRepr(%r)' % self.s

class ClassWithFailingRepr:

    def __repr__(self):
        raise Exception('This should be caught by Repr.repr_instance')

class MyContainer:
    """Helper class for TestRecursiveRepr"""

    def __init__(self, values):
        self.values = list(values)

    def append(self, value):
        self.values.append(value)

    @recursive_repr()
    def __repr__(self):
        return '<' + ', '.join(map(str, self.values)) + '>'

class MyContainer2(MyContainer):

    @recursive_repr('+++')
    def __repr__(self):
        return '<' + ', '.join(map(str, self.values)) + '>'

class MyContainer3:

    def __repr__(self):
        """Test document content"""
        pass
    wrapped = __repr__
    wrapper = recursive_repr()(wrapped)


# --- test body ---
from functools import WRAPPER_ASSIGNMENTS as assigned
wrapped = MyContainer3.wrapped
wrapper = MyContainer3.wrapper
for name in assigned:

    assert getattr(wrapper, name) is getattr(wrapped, name)
print("TestRecursiveRepr::test_assigned_attributes: ok")
