# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "raw_config_parser_test_case_non_standard_delimiters__test_default_case_sensitivity"
# subject = "cpython.test_configparser.RawConfigParserTestCaseNonStandardDelimiters.test_default_case_sensitivity"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_configparser.py::RawConfigParserTestCaseNonStandardDelimiters::test_default_case_sensitivity
"""Auto-ported test: RawConfigParserTestCaseNonStandardDelimiters::test_default_case_sensitivity (CPython 3.12 oracle)."""


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
config_class = configparser.RawConfigParser
delimiters = (':=', '$')
comment_prefixes = ('//', '"')
inline_comment_prefixes = ('//', '"')

def basic_test(cf):
    E = ['Commented Bar', 'Foo Bar', 'Internationalized Stuff', 'Long Line', 'Section\\with$weird%characters[\t', 'Spaces', 'Spacey Bar', 'Spacey Bar From The Beginning', 'Types', 'This One Has A ] In It']
    if allow_no_value:
        E.append('NoValue')
    E.sort()
    F = [('baz', 'qwe'), ('foo', 'bar3')]
    L = cf.sections()
    L.sort()
    eq = self_assertEqual
    eq(L, E)
    L = cf.items('Spacey Bar From The Beginning')
    L.sort()
    eq(L, F)
    L = [section for section in cf]
    L.sort()
    E.append(default_section)
    E.sort()
    eq(L, E)
    L = cf['Spacey Bar From The Beginning'].items()
    L = sorted(list(L))
    eq(L, F)
    L = cf.items()
    L = sorted(list(L))

    assert len(L) == len(E)
    for name, section in L:
        eq(name, section.name)
    eq(cf.defaults(), cf[default_section])
    eq(cf.get('Foo Bar', 'foo'), 'bar1')
    eq(cf.get('Spacey Bar', 'foo'), 'bar2')
    eq(cf.get('Spacey Bar From The Beginning', 'foo'), 'bar3')
    eq(cf.get('Spacey Bar From The Beginning', 'baz'), 'qwe')
    eq(cf.get('Commented Bar', 'foo'), 'bar4')
    eq(cf.get('Commented Bar', 'baz'), 'qwe')
    eq(cf.get('Spaces', 'key with spaces'), 'value')
    eq(cf.get('Spaces', 'another with spaces'), 'splat!')
    eq(cf.getint('Types', 'int'), 42)
    eq(cf.get('Types', 'int'), '42')

    assert abs(cf.getfloat('Types', 'float') - 0.44) < 1e-07
    eq(cf.get('Types', 'float'), '0.44')
    eq(cf.getboolean('Types', 'boolean'), False)
    eq(cf.get('Types', '123'), 'strange but acceptable')
    eq(cf.get('This One Has A ] In It', 'forks'), 'spoons')
    if allow_no_value:
        eq(cf.get('NoValue', 'option-without-value'), None)
    eq(cf.get('Foo Bar', 'foo', fallback='baz'), 'bar1')
    eq(cf.get('Foo Bar', 'foo', vars={'foo': 'baz'}), 'baz')
    try:
        cf.get('No Such Foo Bar', 'foo')
        raise AssertionError('expected configparser.NoSectionError')
    except configparser.NoSectionError:
        pass
    try:
        cf.get('Foo Bar', 'no-such-foo')
        raise AssertionError('expected configparser.NoOptionError')
    except configparser.NoOptionError:
        pass
    eq(cf.get('No Such Foo Bar', 'foo', fallback='baz'), 'baz')
    eq(cf.get('Foo Bar', 'no-such-foo', fallback='baz'), 'baz')
    eq(cf.get('Spacey Bar', 'foo', fallback=None), 'bar2')
    eq(cf.get('No Such Spacey Bar', 'foo', fallback=None), None)
    eq(cf.getint('Types', 'int', fallback=18), 42)
    eq(cf.getint('Types', 'no-such-int', fallback=18), 18)
    eq(cf.getint('Types', 'no-such-int', fallback='18'), '18')
    try:
        cf.getint('Types', 'no-such-int')
        raise AssertionError('expected configparser.NoOptionError')
    except configparser.NoOptionError:
        pass

    assert abs(cf.getfloat('Types', 'float', fallback=0.0) - 0.44) < 1e-07

    assert abs(cf.getfloat('Types', 'no-such-float', fallback=0.0) - 0.0) < 1e-07
    eq(cf.getfloat('Types', 'no-such-float', fallback='0.0'), '0.0')
    try:
        cf.getfloat('Types', 'no-such-float')
        raise AssertionError('expected configparser.NoOptionError')
    except configparser.NoOptionError:
        pass
    eq(cf.getboolean('Types', 'boolean', fallback=True), False)
    eq(cf.getboolean('Types', 'no-such-boolean', fallback='yes'), 'yes')
    eq(cf.getboolean('Types', 'no-such-boolean', fallback=True), True)
    try:
        cf.getboolean('Types', 'no-such-boolean')
        raise AssertionError('expected configparser.NoOptionError')
    except configparser.NoOptionError:
        pass
    eq(cf.getboolean('No Such Types', 'boolean', fallback=True), True)
    if allow_no_value:
        eq(cf.get('NoValue', 'option-without-value', fallback=False), None)
        eq(cf.get('NoValue', 'no-such-option-without-value', fallback=False), False)
    eq(cf['Foo Bar']['foo'], 'bar1')
    eq(cf['Spacey Bar']['foo'], 'bar2')
    section = cf['Spacey Bar From The Beginning']
    eq(section.name, 'Spacey Bar From The Beginning')

    assert section.parser is cf
    try:
        section.name = 'Name is read-only'
        raise AssertionError('expected AttributeError')
    except AttributeError:
        pass
    try:
        section.parser = 'Parser is read-only'
        raise AssertionError('expected AttributeError')
    except AttributeError:
        pass
    eq(section['foo'], 'bar3')
    eq(section['baz'], 'qwe')
    eq(cf['Commented Bar']['foo'], 'bar4')
    eq(cf['Commented Bar']['baz'], 'qwe')
    eq(cf['Spaces']['key with spaces'], 'value')
    eq(cf['Spaces']['another with spaces'], 'splat!')
    eq(cf['Long Line']['foo'], 'this line is much, much longer than my editor\nlikes it.')
    if allow_no_value:
        eq(cf['NoValue']['option-without-value'], None)
    eq(cf['Foo Bar'].get('foo', 'baz'), 'bar1')
    eq(cf['Foo Bar'].get('foo', fallback='baz'), 'bar1')
    eq(cf['Foo Bar'].get('foo', vars={'foo': 'baz'}), 'baz')
    try:
        cf['No Such Foo Bar']['foo']
        raise AssertionError('expected KeyError')
    except KeyError:
        pass
    try:
        cf['Foo Bar']['no-such-foo']
        raise AssertionError('expected KeyError')
    except KeyError:
        pass
    try:
        cf['No Such Foo Bar'].get('foo', fallback='baz')
        raise AssertionError('expected KeyError')
    except KeyError:
        pass
    eq(cf['Foo Bar'].get('no-such-foo', 'baz'), 'baz')
    eq(cf['Foo Bar'].get('no-such-foo', fallback='baz'), 'baz')
    eq(cf['Foo Bar'].get('no-such-foo'), None)
    eq(cf['Spacey Bar'].get('foo', None), 'bar2')
    eq(cf['Spacey Bar'].get('foo', fallback=None), 'bar2')
    try:
        cf['No Such Spacey Bar'].get('foo', None)
        raise AssertionError('expected KeyError')
    except KeyError:
        pass
    eq(cf['Types'].getint('int', 18), 42)
    eq(cf['Types'].getint('int', fallback=18), 42)
    eq(cf['Types'].getint('no-such-int', 18), 18)
    eq(cf['Types'].getint('no-such-int', fallback=18), 18)
    eq(cf['Types'].getint('no-such-int', '18'), '18')
    eq(cf['Types'].getint('no-such-int', fallback='18'), '18')
    eq(cf['Types'].getint('no-such-int'), None)

    assert abs(cf['Types'].getfloat('float', 0.0) - 0.44) < 1e-07

    assert abs(cf['Types'].getfloat('float', fallback=0.0) - 0.44) < 1e-07

    assert abs(cf['Types'].getfloat('no-such-float', 0.0) - 0.0) < 1e-07

    assert abs(cf['Types'].getfloat('no-such-float', fallback=0.0) - 0.0) < 1e-07
    eq(cf['Types'].getfloat('no-such-float', '0.0'), '0.0')
    eq(cf['Types'].getfloat('no-such-float', fallback='0.0'), '0.0')
    eq(cf['Types'].getfloat('no-such-float'), None)
    eq(cf['Types'].getboolean('boolean', True), False)
    eq(cf['Types'].getboolean('boolean', fallback=True), False)
    eq(cf['Types'].getboolean('no-such-boolean', 'yes'), 'yes')
    eq(cf['Types'].getboolean('no-such-boolean', fallback='yes'), 'yes')
    eq(cf['Types'].getboolean('no-such-boolean', True), True)
    eq(cf['Types'].getboolean('no-such-boolean', fallback=True), True)
    eq(cf['Types'].getboolean('no-such-boolean'), None)
    if allow_no_value:
        eq(cf['NoValue'].get('option-without-value', False), None)
        eq(cf['NoValue'].get('option-without-value', fallback=False), None)
        eq(cf['NoValue'].get('no-such-option-without-value', False), False)
        eq(cf['NoValue'].get('no-such-option-without-value', fallback=False), False)
    cf[default_section]['this_value'] = '1'
    cf[default_section]['that_value'] = '2'

    assert cf.remove_section('Spaces')

    assert not cf.has_option('Spaces', 'key with spaces')

    assert not cf.remove_section('Spaces')

    assert not cf.remove_section(default_section)

    assert cf.remove_option('Foo Bar', 'foo')

    assert not cf.has_option('Foo Bar', 'foo')

    assert not cf.remove_option('Foo Bar', 'foo')

    assert cf.has_option('Foo Bar', 'this_value')

    assert not cf.remove_option('Foo Bar', 'this_value')

    assert cf.remove_option(default_section, 'this_value')

    assert not cf.has_option('Foo Bar', 'this_value')

    assert not cf.remove_option(default_section, 'this_value')
    try:
        cf.remove_option('No Such Section', 'foo')
        raise AssertionError('expected configparser.NoSectionError')
    except configparser.NoSectionError as _aR_e:
        import types as _types_aR
        cm = _types_aR.SimpleNamespace(exception=_aR_e)

    assert cm.exception.args == ('No Such Section',)
    eq(cf.get('Long Line', 'foo'), 'this line is much, much longer than my editor\nlikes it.')
    del cf['Types']

    assert not 'Types' in cf
    try:
        del cf['Types']
        raise AssertionError('expected KeyError')
    except KeyError:
        pass
    try:
        del cf[default_section]
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
    del cf['Spacey Bar']['foo']

    assert not 'foo' in cf['Spacey Bar']
    try:
        del cf['Spacey Bar']['foo']
        raise AssertionError('expected KeyError')
    except KeyError:
        pass

    assert 'that_value' in cf['Spacey Bar']
    try:
        del cf['Spacey Bar']['that_value']
        raise AssertionError('expected KeyError')
    except KeyError:
        pass
    del cf[default_section]['that_value']

    assert not 'that_value' in cf['Spacey Bar']
    try:
        del cf[default_section]['that_value']
        raise AssertionError('expected KeyError')
    except KeyError:
        pass
    try:
        del cf['No Such Section']['foo']
        raise AssertionError('expected KeyError')
    except KeyError:
        pass

def check_items_config(expected):
    cf = fromstring('\n            [section]\n            name {0[0]} %(value)s\n            key{0[1]} |%(name)s|\n            getdefault{0[1]} |%(default)s|\n        '.format(delimiters), defaults={'default': '<default>'})
    L = list(cf.items('section', vars={'value': 'value'}))
    L.sort()

    assert L == expected
    try:
        cf.items('no such section')
        raise AssertionError('expected configparser.NoSectionError')
    except configparser.NoSectionError:
        pass

def fromstring(string, defaults=None):
    cf = newconfig(defaults)
    cf.read_string(string)
    return cf

def get_error(cf, exc, section, option):
    try:
        cf.get(section, option)
    except exc as e:
        return e
    else:

        raise AssertionError('expected exception type %s.%s' % (exc.__module__, exc.__qualname__))

def get_interpolation_config():
    return fromstring('[Foo]\nbar{equals}something %(with1)s interpolation (1 step)\nbar9{equals}something %(with9)s lots of interpolation (9 steps)\nbar10{equals}something %(with10)s lots of interpolation (10 steps)\nbar11{equals}something %(with11)s lots of interpolation (11 steps)\nwith11{equals}%(with10)s\nwith10{equals}%(with9)s\nwith9{equals}%(with8)s\nwith8{equals}%(With7)s\nwith7{equals}%(WITH6)s\nwith6{equals}%(with5)s\nWith5{equals}%(with4)s\nWITH4{equals}%(with3)s\nwith3{equals}%(with2)s\nwith2{equals}%(with1)s\nwith1{equals}with\n\n[Mutual Recursion]\nfoo{equals}%(bar)s\nbar{equals}%(foo)s\n\n[Interpolation Error]\nname{equals}%(reference)s\n'.format(equals=delimiters[0]))

def newconfig(defaults=None):
    arguments = dict(defaults=defaults, allow_no_value=allow_no_value, delimiters=delimiters, comment_prefixes=comment_prefixes, inline_comment_prefixes=inline_comment_prefixes, empty_lines_in_values=empty_lines_in_values, dict_type=dict_type, strict=strict, default_section=default_section, interpolation=interpolation)
    instance = config_class(**arguments)
    return instance

def parse_error(cf, exc, src):
    if hasattr(src, 'readline'):
        sio = src
    else:
        sio = io.StringIO(src)
    try:
        cf.read_file(sio)
        raise AssertionError('expected exc')
    except exc as _aR_e:
        import types as _types_aR
        cm = _types_aR.SimpleNamespace(exception=_aR_e)
    return cm.exception
cf = newconfig({'foo': 'Bar'})

assert cf.get(default_section, 'Foo') == 'Bar'
cf = newconfig({'Foo': 'Bar'})

assert cf.get(default_section, 'Foo') == 'Bar'
print("RawConfigParserTestCaseNonStandardDelimiters::test_default_case_sensitivity: ok")
