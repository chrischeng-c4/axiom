# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "reprlib"
# dimension = "behavior"
# case = "repr_tests__test_invalid_indent"
# subject = "cpython.test_reprlib.ReprTests.test_invalid_indent"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_reprlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_reprlib.py::ReprTests::test_invalid_indent
"""Auto-ported test: ReprTests::test_invalid_indent (CPython 3.12 oracle)."""


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
test_object = [1, 'spam', {'eggs': True, 'ham': []}]
test_cases = [(-1, (ValueError, '[Nn]egative|[Pp]ositive')), (-4, (ValueError, '[Nn]egative|[Pp]ositive')), ((), (TypeError, None)), ([], (TypeError, None)), ((4,), (TypeError, None)), ([4], (TypeError, None)), (object(), (TypeError, None))]
for indent, (expected_error, expected_msg) in test_cases:
    r = Repr()
    r.indent = indent
    expected_msg = expected_msg or f'{type(indent)}'
    try:
        r.repr(test_object)
        raise AssertionError('expected expected_error')
    except expected_error as _aR_e:
        import re as _re_aR
        assert _re_aR.search(expected_msg, str(_aR_e))
print("ReprTests::test_invalid_indent: ok")
