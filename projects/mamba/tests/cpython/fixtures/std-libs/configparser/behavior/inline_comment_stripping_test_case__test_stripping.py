# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "inline_comment_stripping_test_case__test_stripping"
# subject = "cpython.test_configparser.InlineCommentStrippingTestCase.test_stripping"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_configparser.py::InlineCommentStrippingTestCase::test_stripping
"""Auto-ported test: InlineCommentStrippingTestCase::test_stripping (CPython 3.12 oracle)."""


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
cfg = configparser.ConfigParser(inline_comment_prefixes=(';', '#', '//'))
cfg.read_string('\n        [section]\n        k1 = v1;still v1\n        k2 = v2 ;a comment\n        k3 = v3 ; also a comment\n        k4 = v4;still v4 ;a comment\n        k5 = v5;still v5 ; also a comment\n        k6 = v6;still v6; and still v6 ;a comment\n        k7 = v7;still v7; and still v7 ; also a comment\n\n        [multiprefix]\n        k1 = v1;still v1 #a comment ; yeah, pretty much\n        k2 = v2 // this already is a comment ; continued\n        k3 = v3;#//still v3# and still v3 ; a comment\n        ')
s = cfg['section']

assert s['k1'] == 'v1;still v1'

assert s['k2'] == 'v2'

assert s['k3'] == 'v3'

assert s['k4'] == 'v4;still v4'

assert s['k5'] == 'v5;still v5'

assert s['k6'] == 'v6;still v6; and still v6'

assert s['k7'] == 'v7;still v7; and still v7'
s = cfg['multiprefix']

assert s['k1'] == 'v1;still v1'

assert s['k2'] == 'v2'

assert s['k3'] == 'v3;#//still v3# and still v3'
print("InlineCommentStrippingTestCase::test_stripping: ok")
