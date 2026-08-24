use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/configparser/coverage_one_hundred_test_case__test_sectionproxy_repr.py`.
#[test]
fn test_gen_behavior_std_libs_configparser_coverage_one_hundred_test_case__test_sectionproxy_repr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "coverage_one_hundred_test_case__test_sectionproxy_repr"
# subject = "cpython.test_configparser.CoverageOneHundredTestCase.test_sectionproxy_repr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_configparser.py::CoverageOneHundredTestCase::test_sectionproxy_repr
"""Auto-ported test: CoverageOneHundredTestCase::test_sectionproxy_repr (CPython 3.12 oracle)."""


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
parser.read_string('\n            [section]\n            key = value\n        ')

assert repr(parser['section']) == '<Section: section>'
print("CoverageOneHundredTestCase::test_sectionproxy_repr: ok")
"###);
    assert_output(&out, r###"CoverageOneHundredTestCase::test_sectionproxy_repr: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/configparser/custom_converter_synthesizes_accessor.py`.
#[test]
fn test_gen_behavior_std_libs_configparser_custom_converter_synthesizes_accessor() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "custom_converter_synthesizes_accessor"
# subject = "configparser.ConfigParser.converters"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.converters: registering converters['decimal'] synthesizes a getdecimal() accessor on both the parser and each section proxy, returning the converted value"""
import configparser
import decimal

parser = configparser.ConfigParser()
parser.converters["decimal"] = decimal.Decimal
parser.read_string("[s1]\none = 1\n[s2]\ntwo = 2\n")

assert "decimal" in parser.converters, "converter registered"

# The accessor exists on the parser ...
assert parser.getdecimal("s1", "one") == decimal.Decimal("1"), "parser.getdecimal s1"
assert parser.getdecimal("s2", "two") == decimal.Decimal("2"), "parser.getdecimal s2"

# ... and on each section proxy.
assert parser["s1"].getdecimal("one") == decimal.Decimal("1"), "proxy getdecimal s1"
assert parser["s2"].getdecimal("two") == decimal.Decimal("2"), "proxy getdecimal s2"

print("custom_converter_synthesizes_accessor OK")
"###);
    assert_output(&out, r###"custom_converter_synthesizes_accessor OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/configparser/default_section_inherited_by_all.py`.
#[test]
fn test_gen_behavior_std_libs_configparser_default_section_inherited_by_all() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "default_section_inherited_by_all"
# subject = "configparser.ConfigParser.get"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.get: options in the DEFAULT section are inherited by every other section (color=blue in DEFAULT is visible from s1 and s2)"""
import configparser

cp = configparser.ConfigParser()
cp.read_string("[DEFAULT]\ncolor = blue\n[s1]\nname = alice\n[s2]\nname = bob\n")

assert cp.get("s1", "color") == "blue", "s1 inherits DEFAULT"
assert cp.get("s2", "color") == "blue", "s2 inherits DEFAULT"
# The section's own option still wins over the inherited default.
assert cp.get("s1", "name") == "alice", "s1 own option"

print("default_section_inherited_by_all OK")
"###);
    assert_output(&out, r###"default_section_inherited_by_all OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/configparser/exception_pickling_test_case__test_error.py`.
#[test]
fn test_gen_behavior_std_libs_configparser_exception_pickling_test_case__test_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "exception_pickling_test_case__test_error"
# subject = "cpython.test_configparser.ExceptionPicklingTestCase.test_error"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_configparser.py::ExceptionPicklingTestCase::test_error
"""Auto-ported test: ExceptionPicklingTestCase::test_error (CPython 3.12 oracle)."""


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
import pickle
e1 = configparser.Error('value')
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    pickled = pickle.dumps(e1, proto)
    e2 = pickle.loads(pickled)

    assert e1.message == e2.message

    assert repr(e1) == repr(e2)
print("ExceptionPicklingTestCase::test_error: ok")
"###);
    assert_output(&out, r###"ExceptionPicklingTestCase::test_error: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/configparser/extended_interpolation_cross_section.py`.
#[test]
fn test_gen_behavior_std_libs_configparser_extended_interpolation_cross_section() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "extended_interpolation_cross_section"
# subject = "configparser.ExtendedInterpolation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ExtendedInterpolation: ExtendedInterpolation resolves ${section:option} cross-section references (${paths:home}/bob -> /home/bob)"""
import configparser

ext = configparser.ConfigParser(interpolation=configparser.ExtendedInterpolation())
ext.read_string("[paths]\nhome = /home\n[user]\ndir = ${paths:home}/bob\n")
assert ext.get("user", "dir") == "/home/bob", f"extended = {ext.get('user', 'dir')!r}"

print("extended_interpolation_cross_section OK")
"###);
    assert_output(&out, r###"extended_interpolation_cross_section OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/configparser/get_fallback_for_missing_option.py`.
#[test]
fn test_gen_behavior_std_libs_configparser_get_fallback_for_missing_option() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "get_fallback_for_missing_option"
# subject = "configparser.ConfigParser.get"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""configparser.ConfigParser.get: get(section, option, fallback=...) returns the fallback instead of raising when the option is absent"""
import configparser

cp = configparser.ConfigParser()
cp.read_string("[section1]\nkey1 = value1\n")

fallback = cp.get("section1", "missing_key", fallback="default_val")
assert fallback == "default_val", f"fallback = {fallback!r}"

# A present option ignores the fallback and returns its real value.
present = cp.get("section1", "key1", fallback="default_val")
assert present == "value1", f"present = {present!r}"

print("get_fallback_for_missing_option OK")
"###);
    assert_output(&out, r###"get_fallback_for_missing_option OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/configparser/getboolean_true_false_variants.py`.
#[test]
fn test_gen_behavior_std_libs_configparser_getboolean_true_false_variants() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "getboolean_true_false_variants"
# subject = "configparser.ConfigParser.getboolean"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""configparser.ConfigParser.getboolean: getboolean recognizes the full true/false vocabulary: yes/on/1/true -> True and no/off/0/false -> False"""
import configparser

cp = configparser.ConfigParser()
cp.read_string("[s]\na=yes\nb=no\nc=on\nd=off\ne=1\nf=0\ng=true\nh=false\n")

assert cp.getboolean("s", "a") is True, "yes=True"
assert cp.getboolean("s", "b") is False, "no=False"
assert cp.getboolean("s", "c") is True, "on=True"
assert cp.getboolean("s", "d") is False, "off=False"
assert cp.getboolean("s", "e") is True, "1=True"
assert cp.getboolean("s", "f") is False, "0=False"
assert cp.getboolean("s", "g") is True, "true=True"
assert cp.getboolean("s", "h") is False, "false=False"

print("getboolean_true_false_variants OK")
"###);
    assert_output(&out, r###"getboolean_true_false_variants OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/configparser/getfloat_returns_float.py`.
#[test]
fn test_gen_behavior_std_libs_configparser_getfloat_returns_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "getfloat_returns_float"
# subject = "configparser.ConfigParser.getfloat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""configparser.ConfigParser.getfloat: getfloat coerces a numeric option string to a Python float (key3 = 3.14 -> 3.14, type float)"""
import configparser

cp = configparser.ConfigParser()
cp.read_string("[section1]\nkey3 = 3.14\n")

f = cp.getfloat("section1", "key3")
assert abs(f - 3.14) < 0.001, f"getfloat = {f!r}"
assert isinstance(f, float), f"getfloat type = {type(f)!r}"

print("getfloat_returns_float OK")
"###);
    assert_output(&out, r###"getfloat_returns_float OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/configparser/getint_returns_int.py`.
#[test]
fn test_gen_behavior_std_libs_configparser_getint_returns_int() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "getint_returns_int"
# subject = "configparser.ConfigParser.getint"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""configparser.ConfigParser.getint: getint coerces a numeric option string to a Python int (key2 = 42 -> 42, type int)"""
import configparser

cp = configparser.ConfigParser()
cp.read_string("[section1]\nkey2 = 42\n")

i = cp.getint("section1", "key2")
assert i == 42, f"getint = {i!r}"
assert isinstance(i, int), f"getint type = {type(i)!r}"

print("getint_returns_int OK")
"###);
    assert_output(&out, r###"getint_returns_int OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/configparser/inline_comment_prefix_stripping.py`.
#[test]
fn test_gen_behavior_std_libs_configparser_inline_comment_prefix_stripping() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "inline_comment_prefix_stripping"
# subject = "configparser.ConfigParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser: with inline_comment_prefixes=(';','#','//') a prefix preceded by whitespace strips the trailing comment, while a prefix glued to the value is kept verbatim; without inline prefixes the marker stays part of the value"""
import configparser

src = (
    "[section]\n"
    "k1 = v1;still v1\n"
    "k2 = v2 ;a comment\n"
    "k3 = v3 ; also a comment\n"
    "k4 = v4;still v4 ;a comment\n"
    "k5 = v5;still v5; and still v5 ;a comment\n"
    "\n"
    "[multi]\n"
    "k1 = v1;still v1 #a comment ; yeah\n"
    "k2 = v2 // this is a comment ; continued\n"
    "k3 = v3;#//still v3# and still v3 ; a comment\n"
)

cfg = configparser.ConfigParser(inline_comment_prefixes=(";", "#", "//"))
cfg.read_string(src)

s = cfg["section"]
assert s["k1"] == "v1;still v1", f"k1 = {s['k1']!r}"  # no space before ; -> kept
assert s["k2"] == "v2", f"k2 = {s['k2']!r}"           # ' ;' strips comment
assert s["k3"] == "v3", f"k3 = {s['k3']!r}"
assert s["k4"] == "v4;still v4", f"k4 = {s['k4']!r}"
assert s["k5"] == "v5;still v5; and still v5", f"k5 = {s['k5']!r}"

m = cfg["multi"]
assert m["k1"] == "v1;still v1", f"multi k1 = {m['k1']!r}"
assert m["k2"] == "v2", f"multi k2 = {m['k2']!r}"  # ' //' strips comment
assert m["k3"] == "v3;#//still v3# and still v3", f"multi k3 = {m['k3']!r}"

# Without inline prefixes the marker stays part of the value.
plain = configparser.ConfigParser()
plain.read_string("[s]\nk = value ; not a comment\n")
assert plain["s"]["k"] == "value ; not a comment", f"plain = {plain['s']['k']!r}"

print("inline_comment_prefix_stripping OK")
"###);
    assert_output(&out, r###"inline_comment_prefix_stripping OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/configparser/interpolation_literal_percent.py`.
#[test]
fn test_gen_behavior_std_libs_configparser_interpolation_literal_percent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "interpolation_literal_percent"
# subject = "configparser.ConfigParser.get"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.get: a doubled %% in a value is an escaped literal percent sign (p = 100%% -> 100%)"""
import configparser

cp = configparser.ConfigParser()
cp.read_string("[s]\np = 100%%\n")
assert cp.get("s", "p") == "100%", f"literal percent = {cp.get('s', 'p')!r}"

print("interpolation_literal_percent OK")
"###);
    assert_output(&out, r###"interpolation_literal_percent OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/configparser/interpolation_raw_bypass.py`.
#[test]
fn test_gen_behavior_std_libs_configparser_interpolation_raw_bypass() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "interpolation_raw_bypass"
# subject = "configparser.ConfigParser.get"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.get: get(..., raw=True) returns the uninterpolated template string verbatim (%(base)s/bob), bypassing substitution"""
import configparser

cp = configparser.ConfigParser()
cp.read_string("[s]\nbase = /home\nfull = %(base)s/bob\n")

assert cp.get("s", "full") == "/home/bob", "interpolated value"
assert cp.get("s", "full", raw=True) == "%(base)s/bob", "raw bypasses interpolation"

print("interpolation_raw_bypass OK")
"###);
    assert_output(&out, r###"interpolation_raw_bypass OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/configparser/interpolation_same_section_substitution.py`.
#[test]
fn test_gen_behavior_std_libs_configparser_interpolation_same_section_substitution() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "interpolation_same_section_substitution"
# subject = "configparser.ConfigParser.get"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.get: basic %(name)s interpolation substitutes another option in the same section (base=/home, full=%(base)s/bob -> /home/bob)"""
import configparser

cp = configparser.ConfigParser()
cp.read_string("[s]\nbase = /home\nfull = %(base)s/bob\n")
assert cp.get("s", "full") == "/home/bob", f"basic = {cp.get('s', 'full')!r}"

print("interpolation_same_section_substitution OK")
"###);
    assert_output(&out, r###"interpolation_same_section_substitution OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/configparser/issue7005_test_case__test_none_as_value_stringified.py`.
#[test]
fn test_gen_behavior_std_libs_configparser_issue7005_test_case__test_none_as_value_stringified() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "issue7005_test_case__test_none_as_value_stringified"
# subject = "cpython.test_configparser.Issue7005TestCase.test_none_as_value_stringified"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_configparser.py::Issue7005TestCase::test_none_as_value_stringified
"""Auto-ported test: Issue7005TestCase::test_none_as_value_stringified (CPython 3.12 oracle)."""


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
expected_output = '[section]\noption = None\n\n'

def prepare(config_class):
    cp = config_class(allow_no_value=False)
    cp.add_section('section')
    cp.set('section', 'option', None)
    sio = io.StringIO()
    cp.write(sio)
    return sio.getvalue()
cp = configparser.ConfigParser(allow_no_value=False)
cp.add_section('section')
try:
    cp.set('section', 'option', None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("Issue7005TestCase::test_none_as_value_stringified: ok")
"###);
    assert_output(&out, r###"Issue7005TestCase::test_none_as_value_stringified: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/configparser/keys_are_case_insensitive.py`.
#[test]
fn test_gen_behavior_std_libs_configparser_keys_are_case_insensitive() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "keys_are_case_insensitive"
# subject = "configparser.ConfigParser.get"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.get: option keys are case-folded by the default optionxform, so MyKey is retrievable as mykey or MYKEY"""
import configparser

cp = configparser.ConfigParser()
cp.read_string("[s]\nMyKey = hello\n")

assert cp.get("s", "mykey") == "hello", "lowercase key lookup"
assert cp.get("s", "MYKEY") == "hello", "uppercase key lookup"
# The stored key is folded to lowercase.
assert cp.options("s") == ["mykey"], f"options = {cp.options('s')!r}"

print("keys_are_case_insensitive OK")
"###);
    assert_output(&out, r###"keys_are_case_insensitive OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/configparser/read_dict_populates_from_nested_dict.py`.
#[test]
fn test_gen_behavior_std_libs_configparser_read_dict_populates_from_nested_dict() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "read_dict_populates_from_nested_dict"
# subject = "configparser.ConfigParser.read_dict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.read_dict: read_dict populates the parser from a nested {section: {key: value}} mapping; values come back as strings and coerce via getint"""
import configparser

cp = configparser.ConfigParser()
cp.read_dict({"db": {"host": "localhost", "port": "5432"}, "cache": {"url": "redis://"}})

assert cp.get("db", "host") == "localhost", f"host = {cp.get('db', 'host')!r}"
assert cp.getint("db", "port") == 5432, f"port = {cp.getint('db', 'port')!r}"
assert cp.get("cache", "url") == "redis://", f"url = {cp.get('cache', 'url')!r}"

print("read_dict_populates_from_nested_dict OK")
"###);
    assert_output(&out, r###"read_dict_populates_from_nested_dict OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/configparser/read_file_accepts_line_iterable.py`.
#[test]
fn test_gen_behavior_std_libs_configparser_read_file_accepts_line_iterable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "read_file_accepts_line_iterable"
# subject = "configparser.ConfigParser.read_file"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.read_file: read_file accepts any iterable of lines (not just a file object) and populates sections; the section is then a member and its keys are readable"""
import configparser

cp = configparser.ConfigParser()
cp.read_file(["[Foo Bar]\n", "foo = newbar\n"])

assert "Foo Bar" in cp, "read_file iterable populates sections"
assert cp["Foo Bar"]["foo"] == "newbar", f"iterable value = {cp['Foo Bar']['foo']!r}"

print("read_file_accepts_line_iterable OK")
"###);
    assert_output(&out, r###"read_file_accepts_line_iterable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/configparser/read_string_populates_sections.py`.
#[test]
fn test_gen_behavior_std_libs_configparser_read_string_populates_sections() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "read_string_populates_sections"
# subject = "configparser.ConfigParser.read_string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.read_string: read_string parses an ini blob; sections() lists non-DEFAULT sections (DEFAULT excluded), has_section / has_option / options reflect the parsed content, and get returns the raw string value"""
import configparser

cp = configparser.ConfigParser()
cp.read_string(
    "[section1]\n"
    "key1 = value1\n"
    "key2 = 42\n"
    "\n"
    "[section2]\n"
    "name = Alice\n"
)

secs = cp.sections()
assert isinstance(secs, list), f"sections type = {type(secs)!r}"
assert "section1" in secs, "section1 present"
assert "section2" in secs, "section2 present"
assert "DEFAULT" not in secs, "DEFAULT not in sections()"

assert cp.has_section("section1"), "has_section true"
assert not cp.has_section("nonexistent"), "has_section false"

assert cp.has_option("section1", "key1"), "has_option true"
assert not cp.has_option("section1", "nokey"), "has_option false"

opts = cp.options("section1")
assert "key1" in opts, "key1 in options"
assert "key2" in opts, "key2 in options"

assert cp.get("section1", "key1") == "value1", f"get = {cp.get('section1', 'key1')!r}"

print("read_string_populates_sections OK")
"###);
    assert_output(&out, r###"read_string_populates_sections OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/configparser/remove_option_and_section.py`.
#[test]
fn test_gen_behavior_std_libs_configparser_remove_option_and_section() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "remove_option_and_section"
# subject = "configparser.ConfigParser.remove_option"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.remove_option: remove_option deletes one key leaving siblings intact; remove_section deletes the whole section so has_section/has_option go False"""
import configparser

cp = configparser.ConfigParser()
cp.read_string("[s]\nk1=v1\nk2=v2\n")

cp.remove_option("s", "k1")
assert not cp.has_option("s", "k1"), "option removed"
assert cp.has_option("s", "k2"), "other option still there"

cp.remove_section("s")
assert not cp.has_section("s"), "section removed"
assert not cp.has_option("s", "k2"), "options gone with the section"

print("remove_option_and_section OK")
"###);
    assert_output(&out, r###"remove_option_and_section OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/configparser/removing_converter_retires_accessor.py`.
#[test]
fn test_gen_behavior_std_libs_configparser_removing_converter_retires_accessor() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "removing_converter_retires_accessor"
# subject = "configparser.ConfigParser.converters"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.converters: deleting converters['decimal'] retires the synthesized getdecimal() accessor on both the parser and its section proxies, so a later call raises AttributeError"""
import configparser
import decimal

parser = configparser.ConfigParser()
parser.converters["decimal"] = decimal.Decimal
parser.read_string("[s1]\none = 1\n")
assert parser.getdecimal("s1", "one") == decimal.Decimal("1"), "accessor present"

# Removing the converter retires the accessor.
del parser.converters["decimal"]
assert "decimal" not in parser.converters, "converter removed"

raised = False
try:
    parser.getdecimal("s1", "one")
except AttributeError:
    raised = True
assert raised, "getdecimal gone after converter removed"

raised_proxy = False
try:
    parser["s1"].getdecimal("one")
except AttributeError:
    raised_proxy = True
assert raised_proxy, "proxy getdecimal gone after converter removed"

print("removing_converter_retires_accessor OK")
"###);
    assert_output(&out, r###"removing_converter_retires_accessor OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/configparser/section_membership_via_in.py`.
#[test]
fn test_gen_behavior_std_libs_configparser_section_membership_via_in() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "section_membership_via_in"
# subject = "configparser.ConfigParser.__contains__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.__contains__: the parser supports `name in parser` section membership; a parsed section is a member, DEFAULT is always a member, and an absent section is not"""
import configparser

cp = configparser.ConfigParser()
cp.read_string("[s]\nk = v\n")

assert "s" in cp, "section membership via in"
assert "DEFAULT" in cp, "DEFAULT always a member"
assert "absent" not in cp, "absent section not a member"

print("section_membership_via_in OK")
"###);
    assert_output(&out, r###"section_membership_via_in OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/configparser/sectionproxy_repr.py`.
#[test]
fn test_gen_behavior_std_libs_configparser_sectionproxy_repr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "sectionproxy_repr"
# subject = "configparser.SectionProxy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.SectionProxy: repr of a section proxy is the fixed '<Section: name>' form"""
import configparser

cp = configparser.ConfigParser()
cp.read_string("[section]\nkey = value\n")

assert repr(cp["section"]) == "<Section: section>", f"proxy repr = {repr(cp['section'])!r}"

print("sectionproxy_repr OK")
"###);
    assert_output(&out, r###"sectionproxy_repr OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/configparser/write_produces_parseable_ini.py`.
#[test]
fn test_gen_behavior_std_libs_configparser_write_produces_parseable_ini() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "write_produces_parseable_ini"
# subject = "configparser.ConfigParser.write"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.write: assigning a section dict then write(buf) emits parseable ini text containing the [section] header and 'key = value' lines"""
import configparser
import io

cp = configparser.ConfigParser()
cp["mysec"] = {"alpha": "1", "beta": "2"}

buf = io.StringIO()
cp.write(buf)
ini_text = buf.getvalue()

assert "[mysec]" in ini_text, "section in output"
assert "alpha = 1" in ini_text, "key=val in output"
assert "beta = 2" in ini_text, "second key=val in output"

# The emitted text round-trips back through the parser.
cp2 = configparser.ConfigParser()
cp2.read_string(ini_text)
assert cp2.get("mysec", "alpha") == "1", "round-trip alpha"
assert cp2.get("mysec", "beta") == "2", "round-trip beta"

print("write_produces_parseable_ini OK")
"###);
    assert_output(&out, r###"write_produces_parseable_ini OK
"###);
}
