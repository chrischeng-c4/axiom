# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "read_file_test_case__test_source_as_bytes"
# subject = "cpython.test_configparser.ReadFileTestCase.test_source_as_bytes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_configparser.py::ReadFileTestCase::test_source_as_bytes
"""Auto-ported test: ReadFileTestCase::test_source_as_bytes (CPython 3.12 oracle)."""


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
"""Issue #18260."""
lines = textwrap.dedent('\n        [badbad]\n        [badbad]').strip().split('\n')
parser = configparser.ConfigParser()
try:
    parser.read_file(lines, source=b'badbad')
    raise AssertionError('expected configparser.DuplicateSectionError')
except configparser.DuplicateSectionError as _aR_e:
    import types as _types_aR
    dse = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(dse.exception) == "While reading from b'badbad' [line  2]: section 'badbad' already exists"
lines = textwrap.dedent('\n        [badbad]\n        bad = bad\n        bad = bad').strip().split('\n')
parser = configparser.ConfigParser()
try:
    parser.read_file(lines, source=b'badbad')
    raise AssertionError('expected configparser.DuplicateOptionError')
except configparser.DuplicateOptionError as _aR_e:
    import types as _types_aR
    dse = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(dse.exception) == "While reading from b'badbad' [line  3]: option 'bad' in section 'badbad' already exists"
lines = textwrap.dedent('\n        [badbad]\n        = bad').strip().split('\n')
parser = configparser.ConfigParser()
try:
    parser.read_file(lines, source=b'badbad')
    raise AssertionError('expected configparser.ParsingError')
except configparser.ParsingError as _aR_e:
    import types as _types_aR
    dse = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(dse.exception) == "Source contains parsing errors: b'badbad'\n\t[line  2]: '= bad'"
lines = textwrap.dedent('\n        [badbad\n        bad = bad').strip().split('\n')
parser = configparser.ConfigParser()
try:
    parser.read_file(lines, source=b'badbad')
    raise AssertionError('expected configparser.MissingSectionHeaderError')
except configparser.MissingSectionHeaderError as _aR_e:
    import types as _types_aR
    dse = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(dse.exception) == "File contains no section headers.\nfile: b'badbad', line: 1\n'[badbad'"
print("ReadFileTestCase::test_source_as_bytes: ok")
