# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "coverage_one_hundred_test_case__test_inconsistent_converters_state"
# subject = "cpython.test_configparser.CoverageOneHundredTestCase.test_inconsistent_converters_state"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_configparser.py::CoverageOneHundredTestCase::test_inconsistent_converters_state
"""Auto-ported test: CoverageOneHundredTestCase::test_inconsistent_converters_state (CPython 3.12 oracle)."""


import collections
import configparser
import io
import os
import textwrap
import unittest
import warnings
from test import support
from test.support import os_helper


class SortedDict(collections.UserDict):

    def items(self):
        return sorted(self.data.items())

    def keys(self):
        return sorted(self.data.keys())

    def values(self):
        return [i[1] for i in self.items()]

    def iteritems(self):
        return iter(self.items())

    def iterkeys(self):
        return iter(self.keys())

    def itervalues(self):
        return iter(self.values())
    __iter__ = iterkeys

class CfgParserTestCaseClass:
    allow_no_value = False
    delimiters = ('=', ':')
    comment_prefixes = (';', '#')
    inline_comment_prefixes = (';', '#')
    empty_lines_in_values = True
    dict_type = configparser._default_dict
    strict = False
    default_section = configparser.DEFAULTSECT
    interpolation = configparser._UNSET

    def newconfig(self, defaults=None):
        arguments = dict(defaults=defaults, allow_no_value=self.allow_no_value, delimiters=self.delimiters, comment_prefixes=self.comment_prefixes, inline_comment_prefixes=self.inline_comment_prefixes, empty_lines_in_values=self.empty_lines_in_values, dict_type=self.dict_type, strict=self.strict, default_section=self.default_section, interpolation=self.interpolation)
        instance = self.config_class(**arguments)
        return instance

    def fromstring(self, string, defaults=None):
        cf = self.newconfig(defaults)
        cf.read_string(string)
        return cf

class FakeFile:

    def __init__(self):
        file_path = support.findfile('cfgparser.1', subdir='configdata')
        with open(file_path, encoding='utf-8') as f:
            self.lines = f.readlines()
            self.lines.reverse()

    def readline(self):
        if len(self.lines):
            return self.lines.pop()
        return ''

def readline_generator(f):
    """As advised in Doc/library/configparser.rst."""
    line = f.readline()
    while line:
        yield line
        line = f.readline()


# --- test body ---
parser = configparser.ConfigParser()
import decimal
parser.converters['decimal'] = decimal.Decimal
parser.read_string('\n            [s1]\n            one = 1\n            [s2]\n            two = 2\n        ')

assert 'decimal' in parser.converters

assert parser.getdecimal('s1', 'one') == 1

assert parser.getdecimal('s2', 'two') == 2

assert parser['s1'].getdecimal('one') == 1

assert parser['s2'].getdecimal('two') == 2
del parser.getdecimal
try:
    parser.getdecimal('s1', 'one')
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass

assert 'decimal' in parser.converters
del parser.converters['decimal']

assert 'decimal' not in parser.converters
try:
    parser.getdecimal('s1', 'one')
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
try:
    parser['s1'].getdecimal('one')
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
try:
    parser['s2'].getdecimal('two')
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
print("CoverageOneHundredTestCase::test_inconsistent_converters_state: ok")
