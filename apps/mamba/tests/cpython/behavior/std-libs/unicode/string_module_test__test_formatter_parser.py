# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicode"
# dimension = "behavior"
# case = "string_module_test__test_formatter_parser"
# subject = "cpython.test_unicode.StringModuleTest.test_formatter_parser"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicode.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unicode.py::StringModuleTest::test_formatter_parser
"""Auto-ported test: StringModuleTest::test_formatter_parser (CPython 3.12 oracle)."""


import _string
import codecs
import datetime
import itertools
import operator
import pickle
import struct
import sys
import textwrap
import unicodedata
import unittest
import warnings
from test.support import warnings_helper
from test import support, string_tests
from test.support.script_helper import assert_python_failure


' Test script for the Unicode implementation.\n\nWritten by Marc-Andre Lemburg (mal@lemburg.com).\n\n(c) Copyright CNRI, All Rights Reserved. NO WARRANTY.\n\n'

try:
    import _testcapi
except ImportError:
    _testcapi = None

def search_function(encoding):

    def decode1(input, errors='strict'):
        return 42

    def encode1(input, errors='strict'):
        return 42

    def encode2(input, errors='strict'):
        return (42, 42)

    def decode2(input, errors='strict'):
        return (42, 42)
    if encoding == 'test.unicode1':
        return (encode1, decode1, None, None)
    elif encoding == 'test.unicode2':
        return (encode2, decode2, None, None)
    else:
        return None

def duplicate_string(text):
    """
    Try to get a fresh clone of the specified text:
    new object with a reference count of 1.

    This is a best-effort: latin1 single letters and the empty
    string ('') are singletons and cannot be cloned.
    """
    return text.encode().decode()

class StrSubclass(str):
    pass

class OtherStrSubclass(str):
    pass

class WithStr:

    def __init__(self, value):
        self.value = value

    def __str__(self):
        return self.value

class WithRepr:

    def __init__(self, value):
        self.value = value

    def __repr__(self):
        return self.value


# --- test body ---
def parse(format):
    return list(_string.formatter_parser(format))
formatter = parse('prefix {2!s}xxx{0:^+10.3f}{obj.attr!s} {z[0]!s:10}')

assert formatter == [('prefix ', '2', '', 's'), ('xxx', '0', '^+10.3f', None), ('', 'obj.attr', '', 's'), (' ', 'z[0]', '10', 's')]
formatter = parse('prefix {} suffix')

assert formatter == [('prefix ', '', '', None), (' suffix', None, None, None)]
formatter = parse('str')

assert formatter == [('str', None, None, None)]
formatter = parse('')

assert formatter == []
formatter = parse('{0}')

assert formatter == [('', '0', '', None)]

try:
    _string.formatter_parser(1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("StringModuleTest::test_formatter_parser: ok")
