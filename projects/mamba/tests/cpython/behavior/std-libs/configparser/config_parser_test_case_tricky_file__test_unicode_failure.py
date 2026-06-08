# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "config_parser_test_case_tricky_file__test_unicode_failure"
# subject = "cpython.test_configparser.ConfigParserTestCaseTrickyFile.test_unicode_failure"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_configparser.py::ConfigParserTestCaseTrickyFile::test_unicode_failure
"""Auto-ported test: ConfigParserTestCaseTrickyFile::test_unicode_failure (CPython 3.12 oracle)."""


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
allow_no_value = False
delimiters = ('=', ':')
comment_prefixes = (';', '#')
inline_comment_prefixes = (';', '#')
empty_lines_in_values = True
dict_type = configparser._default_dict
strict = False
default_section = configparser.DEFAULTSECT
interpolation = configparser._UNSET
config_class = configparser.ConfigParser
delimiters = {'='}
comment_prefixes = {'#'}
allow_no_value = True

def fromstring(string, defaults=None):
    cf = newconfig(defaults)
    cf.read_string(string)
    return cf

def newconfig(defaults=None):
    arguments = dict(defaults=defaults, allow_no_value=allow_no_value, delimiters=delimiters, comment_prefixes=comment_prefixes, inline_comment_prefixes=inline_comment_prefixes, empty_lines_in_values=empty_lines_in_values, dict_type=dict_type, strict=strict, default_section=default_section, interpolation=interpolation)
    instance = config_class(**arguments)
    return instance
tricky = support.findfile('cfgparser.3', subdir='configdata')
cf = newconfig()
try:
    cf.read(tricky, encoding='ascii')
    raise AssertionError('expected UnicodeDecodeError')
except UnicodeDecodeError:
    pass
print("ConfigParserTestCaseTrickyFile::test_unicode_failure: ok")
