# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "blatant_override_converters_test_case__test_instance_assignment"
# subject = "cpython.test_configparser.BlatantOverrideConvertersTestCase.test_instance_assignment"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_configparser.py::BlatantOverrideConvertersTestCase::test_instance_assignment
"""Auto-ported test: BlatantOverrideConvertersTestCase::test_instance_assignment (CPython 3.12 oracle)."""


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
config = '\n        [one]\n        one = false\n        two = false\n        three = long story short\n\n        [two]\n        one = false\n        two = false\n        three = four\n    '

def _test_len(cfg):

    assert len(cfg.converters) == 4

    assert 'boolean' in cfg.converters

    assert 'len' in cfg.converters

    assert 'tysburg' not in cfg.converters

    assert cfg.converters['int'] is None

    assert cfg.converters['float'] is None

    assert cfg.converters['boolean'] is None

    assert cfg.getlen('one', 'one') == 5

    assert cfg.getlen('one', 'two') == 5

    assert cfg.getlen('one', 'three') == 16

    assert cfg.getlen('two', 'one') == 5

    assert cfg.getlen('two', 'two') == 5

    assert cfg.getlen('two', 'three') == 4

    assert cfg.getlen('two', 'four', fallback=0) == 0
    try:
        cfg.getlen('two', 'four')
        raise AssertionError('expected configparser.NoOptionError')
    except configparser.NoOptionError:
        pass

    assert cfg['one'].getlen('one') == 5

    assert cfg['one'].getlen('two') == 5

    assert cfg['one'].getlen('three') == 16

    assert cfg['two'].getlen('one') == 5

    assert cfg['two'].getlen('two') == 5

    assert cfg['two'].getlen('three') == 4

    assert cfg['two'].getlen('four', 0) == 0

    assert cfg['two'].getlen('four') == None
cfg = configparser.ConfigParser()
cfg.getboolean = lambda section, option: True
cfg.getlen = lambda section, option: len(cfg[section][option])
cfg.read_string(config)

assert len(cfg.converters) == 3

assert 'boolean' in cfg.converters

assert 'len' not in cfg.converters

assert cfg.converters['int'] is None

assert cfg.converters['float'] is None

assert cfg.converters['boolean'] is None

assert cfg.getboolean('one', 'one')

assert cfg.getboolean('two', 'two')

assert cfg.getboolean('one', 'two')

assert cfg.getboolean('two', 'one')
cfg.converters['boolean'] = cfg._convert_to_boolean

assert not cfg.getboolean('one', 'one')

assert not cfg.getboolean('two', 'two')

assert not cfg.getboolean('one', 'two')

assert not cfg.getboolean('two', 'one')

assert cfg.getlen('one', 'one') == 5

assert cfg.getlen('one', 'two') == 5

assert cfg.getlen('one', 'three') == 16

assert cfg.getlen('two', 'one') == 5

assert cfg.getlen('two', 'two') == 5

assert cfg.getlen('two', 'three') == 4
try:

    assert cfg['one'].getlen('one') == 5
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
try:

    assert cfg['two'].getlen('one') == 5
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
print("BlatantOverrideConvertersTestCase::test_instance_assignment: ok")
