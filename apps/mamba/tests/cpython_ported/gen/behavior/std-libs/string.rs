use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/string/capwords_basic.py`.
#[test]
fn test_gen_behavior_std_libs_string_capwords_basic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "capwords_basic"
# subject = "string.capwords"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.capwords: capwords title-cases each whitespace-delimited word: 'hello world foo' -> 'Hello World Foo'"""
import string

assert string.capwords("hello world foo") == "Hello World Foo", "capwords basic"
assert string.capwords("hello world") == "Hello World", "capwords two words"
assert string.capwords("abc def ghi") == "Abc Def Ghi", "capwords three words"
print("capwords_basic OK")
"###);
    assert_output(&out, r###"capwords_basic OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/capwords_custom_separator.py`.
#[test]
fn test_gen_behavior_std_libs_string_capwords_custom_separator() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "capwords_custom_separator"
# subject = "string.capwords"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.capwords: a custom separator splits and re-joins on that separator without collapsing: 'ABC-DEF-GHI', '-' -> 'Abc-Def-Ghi' and a tab separator is preserved literally"""
import string

assert string.capwords("hello/world", "/") == "Hello/World", "capwords slash separator"
assert string.capwords("ABC-DEF-GHI", "-") == "Abc-Def-Ghi", "capwords dash separator"
assert string.capwords("ABC-def DEF-ghi GHI", "-") == "Abc-Def def-Ghi ghi", "capwords dash splits only on dash"
assert string.capwords("\taBc\tDeF\t", "\t") == "\tAbc\tDef\t", "capwords tab separator literal"
print("capwords_custom_separator OK")
"###);
    assert_output(&out, r###"capwords_custom_separator OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/capwords_lowercases_rest.py`.
#[test]
fn test_gen_behavior_std_libs_string_capwords_lowercases_rest() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "capwords_lowercases_rest"
# subject = "string.capwords"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.capwords: capwords upper-cases the first letter and lower-cases the rest of each word: 'the quick BROWN fox' -> 'The Quick Brown Fox' and 'ABC DEF GHI' -> 'Abc Def Ghi'"""
import string

assert string.capwords("the quick BROWN fox") == "The Quick Brown Fox", "capwords lowercases rest"
assert string.capwords("ABC DEF GHI") == "Abc Def Ghi", "capwords all-upper input"
print("capwords_lowercases_rest OK")
"###);
    assert_output(&out, r###"capwords_lowercases_rest OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/capwords_strips_and_collapses.py`.
#[test]
fn test_gen_behavior_std_libs_string_capwords_strips_and_collapses() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "capwords_strips_and_collapses"
# subject = "string.capwords"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.capwords: with the default separator capwords strips and collapses runs of whitespace: '   aBc  DeF   ' -> 'Abc Def' and tabs/newlines collapse to single spaces"""
import string

assert string.capwords("  hello  world  ") == "Hello World", "capwords strips/collapses spaces"
assert string.capwords("   aBc  DeF   ") == "Abc Def", "capwords strips edge runs"
assert string.capwords("abc\tdef\nghi") == "Abc Def Ghi", "capwords collapses tabs/newlines"
assert string.capwords("abc\t   def  \nghi") == "Abc Def Ghi", "capwords mixed whitespace"
print("capwords_strips_and_collapses OK")
"###);
    assert_output(&out, r###"capwords_strips_and_collapses OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/constant_char_classes.py`.
#[test]
fn test_gen_behavior_std_libs_string_constant_char_classes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "constant_char_classes"
# subject = "string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string: each constant contains only its own character class: ascii_lowercase all islower, ascii_uppercase all isupper, digits all isdigit"""
import string

assert all(c.islower() for c in string.ascii_lowercase), "all lowercase"
assert all(c.isupper() for c in string.ascii_uppercase), "all uppercase"
assert all(c.isdigit() for c in string.digits), "all digits"
print("constant_char_classes OK")
"###);
    assert_output(&out, r###"constant_char_classes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/constant_compositions.py`.
#[test]
fn test_gen_behavior_std_libs_string_constant_compositions() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "constant_compositions"
# subject = "string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string: the composed constants equal their parts: ascii_letters == lower+upper, hexdigits == digits+'abcdefABCDEF', printable == digits+lower+upper+punctuation+whitespace"""
import string

assert string.ascii_letters == string.ascii_lowercase + string.ascii_uppercase, "ascii_letters composition"
assert string.hexdigits == string.digits + "abcdefABCDEF", "hexdigits composition"
assert string.printable == (
    string.digits
    + string.ascii_lowercase
    + string.ascii_uppercase
    + string.punctuation
    + string.whitespace
), "printable composition"
print("constant_compositions OK")
"###);
    assert_output(&out, r###"constant_compositions OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/constant_letters_digits_disjoint.py`.
#[test]
fn test_gen_behavior_std_libs_string_constant_letters_digits_disjoint() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "constant_letters_digits_disjoint"
# subject = "string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string: the ascii_letters and digits character sets are disjoint (no char is both a letter and a digit)"""
import string

_letter_set = set(string.ascii_letters)
_digit_set = set(string.digits)
assert not (_letter_set & _digit_set), "letters and digits disjoint"
print("constant_letters_digits_disjoint OK")
"###);
    assert_output(&out, r###"constant_letters_digits_disjoint OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/constant_membership.py`.
#[test]
fn test_gen_behavior_std_libs_string_constant_membership() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "constant_membership"
# subject = "string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string: membership probes hold: 'a'/'Z' in ascii_letters, 'a'/'F' in hexdigits, ' ' and newline in whitespace, '!' in punctuation"""
import string

assert "a" in string.ascii_letters and "Z" in string.ascii_letters, "letters membership"
assert "a" in string.hexdigits and "F" in string.hexdigits, "hexdigits membership"
assert " " in string.whitespace and "\n" in string.whitespace, "whitespace membership"
assert "!" in string.punctuation, "punctuation membership"
print("constant_membership OK")
"###);
    assert_output(&out, r###"constant_membership OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/constant_values_exact.py`.
#[test]
fn test_gen_behavior_std_libs_string_constant_values_exact() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "constant_values_exact"
# subject = "string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string: the module constants have their exact documented byte values: whitespace, ascii_lowercase/uppercase, digits, hexdigits, octdigits, punctuation"""
import string

assert string.whitespace == " \t\n\r\x0b\x0c", f"whitespace = {string.whitespace!r}"
assert string.ascii_lowercase == "abcdefghijklmnopqrstuvwxyz", "ascii_lowercase"
assert string.ascii_uppercase == "ABCDEFGHIJKLMNOPQRSTUVWXYZ", "ascii_uppercase"
assert string.digits == "0123456789", f"digits = {string.digits!r}"
assert string.octdigits == "01234567", f"octdigits = {string.octdigits!r}"
assert string.punctuation == "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~", "punctuation value"
print("constant_values_exact OK")
"###);
    assert_output(&out, r###"constant_values_exact OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/formatter_auto_numbering.py`.
#[test]
fn test_gen_behavior_std_libs_string_formatter_auto_numbering() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "formatter_auto_numbering"
# subject = "string.Formatter"
# kind = "semantic"
# xfail = "string.Formatter is a silent dict-stub on mamba; .format() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Formatter: automatic field numbering ({} == {0},{1},...) fills in argument order and supports an auto-numbered nested width: '{:^{}}' with ('bar', 6) -> ' bar  '"""
import string

fmt = string.Formatter()
assert fmt.format("foo{}{}", "bar", 6) == "foobar6", "auto numbering"
assert fmt.format("{:^{}}", "bar", 6) == " bar  ", "auto-numbered nested width"
print("formatter_auto_numbering OK")
"###);
    assert_output(&out, r###"formatter_auto_numbering OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/formatter_conversion_specifiers.py`.
#[test]
fn test_gen_behavior_std_libs_string_formatter_conversion_specifiers() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "formatter_conversion_specifiers"
# subject = "string.Formatter"
# kind = "semantic"
# xfail = "string.Formatter is a silent dict-stub on mamba; .format() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Formatter: the !r/!s/!a conversions apply repr/str/ascii: '{0!a}' escapes chr(255) as '\\xff' and chr(256) as '\\u0100', '{arg!r}' quotes a str"""
import string

fmt = string.Formatter()
assert fmt.format("-{arg!r}-", arg="test") == "-'test'-", "!r conversion"
assert fmt.format("{0!s}", "test") == "test", "!s conversion"
assert fmt.format("{0!a}", chr(255)) == "'\\xff'", "!a escapes non-ascii"
assert fmt.format("{0!a}", chr(256)) == "'\\u0100'", "!a escapes wide"
print("formatter_conversion_specifiers OK")
"###);
    assert_output(&out, r###"formatter_conversion_specifiers OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/formatter_index_and_attribute_lookup.py`.
#[test]
fn test_gen_behavior_std_libs_string_formatter_index_and_attribute_lookup() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "formatter_index_and_attribute_lookup"
# subject = "string.Formatter"
# kind = "semantic"
# xfail = "string.Formatter is a silent dict-stub on mamba; .format() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Formatter: {0[i]} indexes into a sequence argument and {0.attr} reads an attribute: '{0[2]}{0[0]}' over ['eggs','and','spam'] -> 'spameggs'"""
import string

fmt = string.Formatter()
assert fmt.format("{0[2]}{0[0]}", ["eggs", "and", "spam"]) == "spameggs", "index lookup"


class AnyAttr:
    def __getattr__(self, attr):
        return attr


assert fmt.format("{0.lumber}{0.jack}", AnyAttr()) == "lumberjack", "attr lookup"
print("formatter_index_and_attribute_lookup OK")
"###);
    assert_output(&out, r###"formatter_index_and_attribute_lookup OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/formatter_keyword_fields.py`.
#[test]
fn test_gen_behavior_std_libs_string_formatter_keyword_fields() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "formatter_keyword_fields"
# subject = "string.Formatter"
# kind = "semantic"
# xfail = "string.Formatter is a silent dict-stub on mamba; .format() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Formatter: named/keyword fields resolve from kwargs: Formatter().format('-{arg}-', arg='test') == '-test-'"""
import string

fmt = string.Formatter()
assert fmt.format("-{arg}-", arg="test") == "-test-", "keyword field"
assert fmt.format("{first}{second}", first="a", second="b") == "ab", "two keyword fields"
print("formatter_keyword_fields OK")
"###);
    assert_output(&out, r###"formatter_keyword_fields OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/formatter_mixed_numbering_raises.py`.
#[test]
fn test_gen_behavior_std_libs_string_formatter_mixed_numbering_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "formatter_mixed_numbering_raises"
# subject = "string.Formatter"
# kind = "semantic"
# xfail = "string.Formatter is a silent dict-stub on mamba; .format() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Formatter: mixing automatic ({}) and manual ({1}) field numbering in one template raises ValueError, for both 'foo{1}{}' and 'foo{}{1}'"""
import string

fmt = string.Formatter()
for bad in ("foo{1}{}", "foo{}{1}"):
    _raised = False
    try:
        fmt.format(bad, "bar", 6)
    except ValueError:
        _raised = True
    assert _raised, f"expected ValueError for {bad!r}"
print("formatter_mixed_numbering_raises OK")
"###);
    assert_output(&out, r###"formatter_mixed_numbering_raises OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/formatter_override_check_unused_args.py`.
#[test]
fn test_gen_behavior_std_libs_string_formatter_override_check_unused_args() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "formatter_override_check_unused_args"
# subject = "string.Formatter"
# kind = "semantic"
# xfail = "string.Formatter subclassing relies on the format engine that is a silent dict-stub on mamba (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Formatter: overriding check_unused_args() rejects any unconsumed argument: a StrictFormatter accepts a template using all args but raises ValueError when an arg is left unused"""
import string


class StrictFormatter(string.Formatter):
    def check_unused_args(self, used_args, args, kwargs):
        unused = set(kwargs.keys())
        unused.update(range(len(args)))
        for arg in used_args:
            unused.remove(arg)
        if unused:
            raise ValueError("unused arguments")


strict = StrictFormatter()
assert strict.format("{0}{i}{1}", 10, 20, i=100) == "1010020", "all args used"
for args, kwargs in [((10, 20), {}), ((10,), {"i": 100})]:
    _raised = False
    try:
        strict.format("{0}", *args, **kwargs)
    except ValueError:
        _raised = True
    assert _raised, "expected ValueError for unused args"
print("formatter_override_check_unused_args OK")
"###);
    assert_output(&out, r###"formatter_override_check_unused_args OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/formatter_override_convert_field.py`.
#[test]
fn test_gen_behavior_std_libs_string_formatter_override_convert_field() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "formatter_override_convert_field"
# subject = "string.Formatter"
# kind = "semantic"
# xfail = "string.Formatter subclassing relies on the format engine that is a silent dict-stub on mamba (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Formatter: overriding convert_field() adds a custom '!x' conversion while delegating others to super: XFormatter().format('{0!r}:{0!x}', 'foo', 'foo') == "'foo':None" """
import string


class XFormatter(string.Formatter):
    def convert_field(self, value, conversion):
        if conversion == "x":
            return None
        return super().convert_field(value, conversion)


assert XFormatter().format("{0!r}:{0!x}", "foo", "foo") == "'foo':None", "convert_field override"
print("formatter_override_convert_field OK")
"###);
    assert_output(&out, r###"formatter_override_convert_field OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/formatter_override_format_field.py`.
#[test]
fn test_gen_behavior_std_libs_string_formatter_override_format_field() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "formatter_override_format_field"
# subject = "string.Formatter"
# kind = "semantic"
# xfail = "string.Formatter subclassing relies on the format engine that is a silent dict-stub on mamba (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Formatter: overriding format_field() can transform the value before formatting: CallFormatter calling the value -> format('*{0}*', lambda: 'result') == '*result*'"""
import string


class CallFormatter(string.Formatter):
    def format_field(self, value, format_spec):
        return format(value(), format_spec)


assert CallFormatter().format("*{0}*", lambda: "result") == "*result*", "format_field override"
print("formatter_override_format_field OK")
"###);
    assert_output(&out, r###"formatter_override_format_field OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/formatter_override_parse.py`.
#[test]
fn test_gen_behavior_std_libs_string_formatter_override_parse() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "formatter_override_parse"
# subject = "string.Formatter"
# kind = "semantic"
# xfail = "string.Formatter subclassing relies on the format engine that is a silent dict-stub on mamba (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Formatter: overriding parse() defines a custom '|'-delimited field syntax: BarFormatter().format('*|+0:^10s|*', 'foo') == '*   foo    *'"""
import string


class BarFormatter(string.Formatter):
    def parse(self, format_string):
        for field in format_string.split("|"):
            if field[0] == "+":
                field_name, _, format_spec = field[1:].partition(":")
                yield ("", field_name, format_spec, None)
            else:
                yield (field, None, None, None)


assert BarFormatter().format("*|+0:^10s|*", "foo") == "*   foo    *", "parse override"
print("formatter_override_parse OK")
"###);
    assert_output(&out, r###"formatter_override_parse OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/formatter_positional_fields.py`.
#[test]
fn test_gen_behavior_std_libs_string_formatter_positional_fields() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "formatter_positional_fields"
# subject = "string.Formatter"
# kind = "semantic"
# xfail = "string.Formatter is a silent dict-stub on mamba; .format() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Formatter: Formatter.format passes plain text through and fills explicit positional fields, reusing an index across the template: 'foo{1}{0}-{1}' with ('bar', 6) -> 'foo6bar-6'"""
import string

fmt = string.Formatter()
assert fmt.format("foo") == "foo", "plain text"
assert fmt.format("foo{0}", "bar") == "foobar", "positional 0"
assert fmt.format("foo{1}{0}-{1}", "bar", 6) == "foo6bar-6", "reused positional"
print("formatter_positional_fields OK")
"###);
    assert_output(&out, r###"formatter_positional_fields OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/module_test__test_attrs.py`.
#[test]
fn test_gen_behavior_std_libs_string_module_test__test_attrs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "module_test__test_attrs"
# subject = "cpython.test_string.ModuleTest.test_attrs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::ModuleTest::test_attrs
"""Auto-ported test: ModuleTest::test_attrs (CPython 3.12 oracle)."""


import unittest
import string
from string import Template


class Bag:
    pass

class Mapping:

    def __getitem__(self, name):
        obj = self
        for part in name.split('.'):
            try:
                obj = getattr(obj, part)
            except AttributeError:
                raise KeyError(name)
        return obj


# --- test body ---

assert string.whitespace == ' \t\n\r\x0b\x0c'

assert string.ascii_lowercase == 'abcdefghijklmnopqrstuvwxyz'

assert string.ascii_uppercase == 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'

assert string.ascii_letters == string.ascii_lowercase + string.ascii_uppercase

assert string.digits == '0123456789'

assert string.hexdigits == string.digits + 'abcdefABCDEF'

assert string.octdigits == '01234567'

assert string.punctuation == '!"#$%&\'()*+,-./:;<=>?@[\\]^_`{|}~'

assert string.printable == string.digits + string.ascii_lowercase + string.ascii_uppercase + string.punctuation + string.whitespace
print("ModuleTest::test_attrs: ok")
"###);
    assert_output(&out, r###"ModuleTest::test_attrs: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/module_test__test_auto_numbering.py`.
#[test]
fn test_gen_behavior_std_libs_string_module_test__test_auto_numbering() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "module_test__test_auto_numbering"
# subject = "cpython.test_string.ModuleTest.test_auto_numbering"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::ModuleTest::test_auto_numbering
"""Auto-ported test: ModuleTest::test_auto_numbering (CPython 3.12 oracle)."""


import unittest
import string
from string import Template


class Bag:
    pass

class Mapping:

    def __getitem__(self, name):
        obj = self
        for part in name.split('.'):
            try:
                obj = getattr(obj, part)
            except AttributeError:
                raise KeyError(name)
        return obj


# --- test body ---
fmt = string.Formatter()

assert fmt.format('foo{}{}', 'bar', 6) == 'foo{}{}'.format('bar', 6)

assert fmt.format('foo{1}{num}{1}', None, 'bar', num=6) == 'foo{1}{num}{1}'.format(None, 'bar', num=6)

assert fmt.format('{:^{}}', 'bar', 6) == '{:^{}}'.format('bar', 6)

assert fmt.format('{:^{}} {}', 'bar', 6, 'X') == '{:^{}} {}'.format('bar', 6, 'X')

assert fmt.format('{:^{pad}}{}', 'foo', 'bar', pad=6) == '{:^{pad}}{}'.format('foo', 'bar', pad=6)
try:
    fmt.format('foo{1}{}', 'bar', 6)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    fmt.format('foo{}{1}', 'bar', 6)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("ModuleTest::test_auto_numbering: ok")
"###);
    assert_output(&out, r###"ModuleTest::test_auto_numbering: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/module_test__test_capwords.py`.
#[test]
fn test_gen_behavior_std_libs_string_module_test__test_capwords() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "module_test__test_capwords"
# subject = "cpython.test_string.ModuleTest.test_capwords"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::ModuleTest::test_capwords
"""Auto-ported test: ModuleTest::test_capwords (CPython 3.12 oracle)."""


import unittest
import string
from string import Template


class Bag:
    pass

class Mapping:

    def __getitem__(self, name):
        obj = self
        for part in name.split('.'):
            try:
                obj = getattr(obj, part)
            except AttributeError:
                raise KeyError(name)
        return obj


# --- test body ---

assert string.capwords('abc def ghi') == 'Abc Def Ghi'

assert string.capwords('abc\tdef\nghi') == 'Abc Def Ghi'

assert string.capwords('abc\t   def  \nghi') == 'Abc Def Ghi'

assert string.capwords('ABC DEF GHI') == 'Abc Def Ghi'

assert string.capwords('ABC-DEF-GHI', '-') == 'Abc-Def-Ghi'

assert string.capwords('ABC-def DEF-ghi GHI') == 'Abc-def Def-ghi Ghi'

assert string.capwords('   aBc  DeF   ') == 'Abc Def'

assert string.capwords('\taBc\tDeF\t') == 'Abc Def'

assert string.capwords('\taBc\tDeF\t', '\t') == '\tAbc\tDef\t'
print("ModuleTest::test_capwords: ok")
"###);
    assert_output(&out, r###"ModuleTest::test_capwords: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/module_test__test_conversion_specifiers.py`.
#[test]
fn test_gen_behavior_std_libs_string_module_test__test_conversion_specifiers() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "module_test__test_conversion_specifiers"
# subject = "cpython.test_string.ModuleTest.test_conversion_specifiers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::ModuleTest::test_conversion_specifiers
"""Auto-ported test: ModuleTest::test_conversion_specifiers (CPython 3.12 oracle)."""


import unittest
import string
from string import Template


class Bag:
    pass

class Mapping:

    def __getitem__(self, name):
        obj = self
        for part in name.split('.'):
            try:
                obj = getattr(obj, part)
            except AttributeError:
                raise KeyError(name)
        return obj


# --- test body ---
fmt = string.Formatter()

assert fmt.format('-{arg!r}-', arg='test') == "-'test'-"

assert fmt.format('{0!s}', 'test') == 'test'

try:
    fmt.format('{0!h}', 'test')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert fmt.format('{0!a}', 42) == '42'

assert fmt.format('{0!a}', string.ascii_letters) == "'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ'"

assert fmt.format('{0!a}', chr(255)) == "'\\xff'"

assert fmt.format('{0!a}', chr(256)) == "'\\u0100'"
print("ModuleTest::test_conversion_specifiers: ok")
"###);
    assert_output(&out, r###"ModuleTest::test_conversion_specifiers: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/module_test__test_index_lookup.py`.
#[test]
fn test_gen_behavior_std_libs_string_module_test__test_index_lookup() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "module_test__test_index_lookup"
# subject = "cpython.test_string.ModuleTest.test_index_lookup"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::ModuleTest::test_index_lookup
"""Auto-ported test: ModuleTest::test_index_lookup (CPython 3.12 oracle)."""


import unittest
import string
from string import Template


class Bag:
    pass

class Mapping:

    def __getitem__(self, name):
        obj = self
        for part in name.split('.'):
            try:
                obj = getattr(obj, part)
            except AttributeError:
                raise KeyError(name)
        return obj


# --- test body ---
fmt = string.Formatter()
lookup = ['eggs', 'and', 'spam']

assert fmt.format('{0[2]}{0[0]}', lookup) == 'spameggs'
try:
    fmt.format('{0[2]}{0[0]}', [])
    raise AssertionError('expected IndexError')
except IndexError:
    pass
try:
    fmt.format('{0[2]}{0[0]}', {})
    raise AssertionError('expected KeyError')
except KeyError:
    pass
print("ModuleTest::test_index_lookup: ok")
"###);
    assert_output(&out, r###"ModuleTest::test_index_lookup: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/module_test__test_name_lookup.py`.
#[test]
fn test_gen_behavior_std_libs_string_module_test__test_name_lookup() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "module_test__test_name_lookup"
# subject = "cpython.test_string.ModuleTest.test_name_lookup"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::ModuleTest::test_name_lookup
"""Auto-ported test: ModuleTest::test_name_lookup (CPython 3.12 oracle)."""


import unittest
import string
from string import Template


class Bag:
    pass

class Mapping:

    def __getitem__(self, name):
        obj = self
        for part in name.split('.'):
            try:
                obj = getattr(obj, part)
            except AttributeError:
                raise KeyError(name)
        return obj


# --- test body ---
fmt = string.Formatter()

class AnyAttr:

    def __getattr__(self, attr):
        return attr
x = AnyAttr()

assert fmt.format('{0.lumber}{0.jack}', x) == 'lumberjack'
try:
    fmt.format('{0.lumber}{0.jack}', '')
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
print("ModuleTest::test_name_lookup: ok")
"###);
    assert_output(&out, r###"ModuleTest::test_name_lookup: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/module_test__test_override_convert_field.py`.
#[test]
fn test_gen_behavior_std_libs_string_module_test__test_override_convert_field() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "module_test__test_override_convert_field"
# subject = "cpython.test_string.ModuleTest.test_override_convert_field"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::ModuleTest::test_override_convert_field
"""Auto-ported test: ModuleTest::test_override_convert_field (CPython 3.12 oracle)."""


import unittest
import string
from string import Template


class Bag:
    pass

class Mapping:

    def __getitem__(self, name):
        obj = self
        for part in name.split('.'):
            try:
                obj = getattr(obj, part)
            except AttributeError:
                raise KeyError(name)
        return obj


# --- test body ---
class XFormatter(string.Formatter):

    def convert_field(self, value, conversion):
        if conversion == 'x':
            return None
        return super().convert_field(value, conversion)
fmt = XFormatter()

assert fmt.format('{0!r}:{0!x}', 'foo', 'foo') == "'foo':None"
print("ModuleTest::test_override_convert_field: ok")
"###);
    assert_output(&out, r###"ModuleTest::test_override_convert_field: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/module_test__test_override_format_field.py`.
#[test]
fn test_gen_behavior_std_libs_string_module_test__test_override_format_field() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "module_test__test_override_format_field"
# subject = "cpython.test_string.ModuleTest.test_override_format_field"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::ModuleTest::test_override_format_field
"""Auto-ported test: ModuleTest::test_override_format_field (CPython 3.12 oracle)."""


import unittest
import string
from string import Template


class Bag:
    pass

class Mapping:

    def __getitem__(self, name):
        obj = self
        for part in name.split('.'):
            try:
                obj = getattr(obj, part)
            except AttributeError:
                raise KeyError(name)
        return obj


# --- test body ---
class CallFormatter(string.Formatter):

    def format_field(self, value, format_spec):
        return format(value(), format_spec)
fmt = CallFormatter()

assert fmt.format('*{0}*', lambda: 'result') == '*result*'
print("ModuleTest::test_override_format_field: ok")
"###);
    assert_output(&out, r###"ModuleTest::test_override_format_field: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/module_test__test_override_parse.py`.
#[test]
fn test_gen_behavior_std_libs_string_module_test__test_override_parse() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "module_test__test_override_parse"
# subject = "cpython.test_string.ModuleTest.test_override_parse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::ModuleTest::test_override_parse
"""Auto-ported test: ModuleTest::test_override_parse (CPython 3.12 oracle)."""


import unittest
import string
from string import Template


class Bag:
    pass

class Mapping:

    def __getitem__(self, name):
        obj = self
        for part in name.split('.'):
            try:
                obj = getattr(obj, part)
            except AttributeError:
                raise KeyError(name)
        return obj


# --- test body ---
class BarFormatter(string.Formatter):

    def parse(self, format_string):
        for field in format_string.split('|'):
            if field[0] == '+':
                field_name, _, format_spec = field[1:].partition(':')
                yield ('', field_name, format_spec, None)
            else:
                yield (field, None, None, None)
fmt = BarFormatter()

assert fmt.format('*|+0:^10s|*', 'foo') == '*   foo    *'
print("ModuleTest::test_override_parse: ok")
"###);
    assert_output(&out, r###"ModuleTest::test_override_parse: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/module_test__test_vformat_recursion_limit.py`.
#[test]
fn test_gen_behavior_std_libs_string_module_test__test_vformat_recursion_limit() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "module_test__test_vformat_recursion_limit"
# subject = "cpython.test_string.ModuleTest.test_vformat_recursion_limit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::ModuleTest::test_vformat_recursion_limit
"""Auto-ported test: ModuleTest::test_vformat_recursion_limit (CPython 3.12 oracle)."""


import unittest
import string
from string import Template


class Bag:
    pass

class Mapping:

    def __getitem__(self, name):
        obj = self
        for part in name.split('.'):
            try:
                obj = getattr(obj, part)
            except AttributeError:
                raise KeyError(name)
        return obj


# --- test body ---
fmt = string.Formatter()
args = ()
kwargs = dict(i=100)
try:
    fmt._vformat('{i}', args, kwargs, set(), -1)
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import types as _types_aR
    err = _types_aR.SimpleNamespace(exception=_aR_e)

assert 'recursion' in str(err.exception)
print("ModuleTest::test_vformat_recursion_limit: ok")
"###);
    assert_output(&out, r###"ModuleTest::test_vformat_recursion_limit: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/str_case_methods.py`.
#[test]
fn test_gen_behavior_std_libs_string_str_case_methods() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "str_case_methods"
# subject = "str.upper"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.upper: case methods upper/lower/title/capitalize/swapcase produce the documented results, e.g. 'Hello World'.swapcase()=='hELLO wORLD'"""
import builtins  # noqa: F401

assert "hello".upper() == "HELLO", "upper"
assert "HELLO".lower() == "hello", "lower"
assert "hello world".title() == "Hello World", "title"
assert "hello world".capitalize() == "Hello world", "capitalize"
assert "Hello World".swapcase() == "hELLO wORLD", "swapcase"
print("str_case_methods OK")
"###);
    assert_output(&out, r###"str_case_methods OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/str_find_and_count.py`.
#[test]
fn test_gen_behavior_std_libs_string_str_find_and_count() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "str_find_and_count"
# subject = "str.find"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.find: find returns the first index or -1, count returns occurrences: 'hello world'.find('world')==6, .find('xyz')==-1, 'hello'.count('l')==2"""
import builtins  # noqa: F401

assert "hello world".find("world") == 6, "find hit index"
assert "hello world".find("xyz") == -1, "find miss is -1"
assert "hello".count("l") == 2, "count l"
assert "aaa".count("a") == 3, "count a"
print("str_find_and_count OK")
"###);
    assert_output(&out, r###"str_find_and_count OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/str_predicate_methods.py`.
#[test]
fn test_gen_behavior_std_libs_string_str_predicate_methods() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "str_predicate_methods"
# subject = "str.isdigit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.isdigit: the character-class predicates isdigit/isalpha/isalnum classify strings: '123'.isdigit() is True, 'abc 123'.isalnum() is False"""
import builtins  # noqa: F401

assert "123".isdigit() == True, "isdigit digits"
assert "abc".isdigit() == False, "isdigit letters"
assert "abc".isalpha() == True, "isalpha letters"
assert "123".isalpha() == False, "isalpha digits"
assert "abc123".isalnum() == True, "isalnum alphanumeric"
assert "abc 123".isalnum() == False, "isalnum with space"
print("str_predicate_methods OK")
"###);
    assert_output(&out, r###"str_predicate_methods OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/str_replace.py`.
#[test]
fn test_gen_behavior_std_libs_string_str_replace() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "str_replace"
# subject = "str.replace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.replace: str.replace substitutes every occurrence of the substring: 'hello world'.replace('world','mamba')=='hello mamba', 'aaa'.replace('a','b')=='bbb'"""
import builtins  # noqa: F401

assert "hello world".replace("world", "mamba") == "hello mamba", "replace word"
assert "aaa".replace("a", "b") == "bbb", "replace all chars"
print("str_replace OK")
"###);
    assert_output(&out, r###"str_replace OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/str_split_and_join.py`.
#[test]
fn test_gen_behavior_std_libs_string_str_split_and_join() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "str_split_and_join"
# subject = "str.split"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.split: str.split on whitespace and on a delimiter, plus the inverse str.join: 'hello world foo'.split()==['hello','world','foo'], ','.join(...) round-trips"""
import builtins  # noqa: F401

assert "hello world foo".split() == ["hello", "world", "foo"], "split on whitespace"
assert "a,b,c".split(",") == ["a", "b", "c"], "split on comma"
assert "a,,b".split(",") == ["a", "", "b"], "split keeps empty field"
assert ",".join(["a", "b", "c"]) == "a,b,c", "join with comma"
assert " ".join(["hello", "world"]) == "hello world", "join with space"
assert "".join(["a", "b", "c"]) == "abc", "join with empty"
print("str_split_and_join OK")
"###);
    assert_output(&out, r###"str_split_and_join OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/str_startswith_endswith.py`.
#[test]
fn test_gen_behavior_std_libs_string_str_startswith_endswith() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "str_startswith_endswith"
# subject = "str.startswith"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.startswith: startswith/endswith report prefix/suffix membership as bools: 'hello'.startswith('hel') is True, 'hello'.endswith('xyz') is False"""
import builtins  # noqa: F401

assert "hello".startswith("hel") == True, "startswith match"
assert "hello".startswith("xyz") == False, "startswith no match"
assert "hello".endswith("llo") == True, "endswith match"
assert "hello".endswith("xyz") == False, "endswith no match"
print("str_startswith_endswith OK")
"###);
    assert_output(&out, r###"str_startswith_endswith OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/str_strip_variants.py`.
#[test]
fn test_gen_behavior_std_libs_string_str_strip_variants() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "str_strip_variants"
# subject = "str.strip"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.strip: strip/lstrip/rstrip trim leading/trailing/both whitespace: '  hello  '.strip()=='hello', .lstrip()=='hello  ', .rstrip()=='  hello'"""
import builtins  # noqa: F401

assert "  hello  ".strip() == "hello", "strip both"
assert "  hello  ".lstrip() == "hello  ", "lstrip left"
assert "  hello  ".rstrip() == "  hello", "rstrip right"
print("str_strip_variants OK")
"###);
    assert_output(&out, r###"str_strip_variants OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/str_zfill.py`.
#[test]
fn test_gen_behavior_std_libs_string_str_zfill() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "str_zfill"
# subject = "str.zfill"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.zfill: zfill left-pads with zeros keeping a leading sign: '42'.zfill(5)=='00042' and '-42'.zfill(5)=='-0042'"""
import builtins  # noqa: F401

assert "42".zfill(5) == "00042", "zfill pads"
assert "-42".zfill(5) == "-0042", "zfill keeps sign"
print("str_zfill OK")
"###);
    assert_output(&out, r###"str_zfill OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/template_braced_substitution.py`.
#[test]
fn test_gen_behavior_std_libs_string_template_braced_substitution() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "template_braced_substitution"
# subject = "string.Template"
# kind = "semantic"
# xfail = "string.Template is a silent dict-stub on mamba; .substitute() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Template: ${braced} fields delimit adjacent text: Template('${prefix}ing').substitute(prefix='walk') == 'walking' and mixed $who/${what} forms resolve"""
import string

assert string.Template("${prefix}ing").substitute(prefix="walk") == "walking", "braced sub"
s = string.Template("$who likes ${what} for ${meal}")
d = {"who": "tim", "what": "ham", "meal": "dinner"}
assert s.substitute(d) == "tim likes ham for dinner", "braced fields"
print("template_braced_substitution OK")
"###);
    assert_output(&out, r###"template_braced_substitution OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/template_dollar_escape.py`.
#[test]
fn test_gen_behavior_std_libs_string_template_dollar_escape() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "template_dollar_escape"
# subject = "string.Template"
# kind = "semantic"
# xfail = "string.Template is a silent dict-stub on mamba; .substitute() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Template: a doubled $$ is a literal '$': Template('Cost: $$100').substitute() == 'Cost: $100' and a mixed '$$100' escape inside named fields stays literal"""
import string

assert string.Template("Cost: $$100").substitute() == "Cost: $100", "literal $ escape"
s = string.Template("$who likes to eat a bag of $what worth $$100")
got = s.substitute({"who": "tim", "what": "ham"})
assert got == "tim likes to eat a bag of ham worth $100", f"mixed = {got!r}"
print("template_dollar_escape OK")
"###);
    assert_output(&out, r###"template_dollar_escape OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/template_identifier_grammar.py`.
#[test]
fn test_gen_behavior_std_libs_string_template_identifier_grammar() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "template_identifier_grammar"
# subject = "string.Template"
# kind = "semantic"
# xfail = "string.Template is a silent dict-stub on mamba; .substitute() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Template: identifiers allow digits/underscores and are case-sensitive: '$_wh0_ ${_w_h_a_t_} ${mea1}' and upper-case '$WHO ${WHAT}' both resolve from their mappings"""
import string

# Identifiers may contain digits and underscores.
s = string.Template("$_wh0_ likes ${_w_h_a_t_} for ${mea1}")
d = {"_wh0_": "tim", "_w_h_a_t_": "ham", "mea1": "dinner"}
assert s.substitute(d) == "tim likes ham for dinner", "non-letter identifiers"
# Identifiers are case-sensitive and may be upper-case.
s = string.Template("$WHO likes ${WHAT} for ${MEAL}")
d = {"WHO": "tim", "WHAT": "ham", "MEAL": "dinner"}
assert s.substitute(d) == "tim likes ham for dinner", "upper-case identifiers"
print("template_identifier_grammar OK")
"###);
    assert_output(&out, r###"template_identifier_grammar OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/template_named_substitution.py`.
#[test]
fn test_gen_behavior_std_libs_string_template_named_substitution() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "template_named_substitution"
# subject = "string.Template"
# kind = "semantic"
# xfail = "string.Template is a silent dict-stub on mamba; .substitute() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Template: $name fields are replaced from keyword args and the mapping: Template('$first $last').substitute(first='John', last='Doe') == 'John Doe'"""
import string

# Keyword form.
t = string.Template("$first $last")
assert t.substitute(first="John", last="Doe") == "John Doe", "template kwargs sub"
# Mapping form, and a numeric example.
assert string.Template("$x + $y = $z").substitute({"x": 1, "y": 2, "z": 3}) == "1 + 2 = 3", "template mapping sub"
print("template_named_substitution OK")
"###);
    assert_output(&out, r###"template_named_substitution OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/template_safe_substitute_keeps_unknown.py`.
#[test]
fn test_gen_behavior_std_libs_string_template_safe_substitute_keeps_unknown() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "template_safe_substitute_keeps_unknown"
# subject = "string.Template"
# kind = "semantic"
# xfail = "string.Template is a silent dict-stub on mamba; .safe_substitute() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Template: safe_substitute fills known fields and leaves unknown placeholders literally in place instead of raising: 'Hello $name, $greeting!' with name only keeps '$greeting'"""
import string

t = string.Template("Hello $name, $greeting!")
result = t.safe_substitute(name="Alice")
assert "Alice" in result, f"safe name = {result!r}"
assert "$greeting" in result, f"safe missing kept = {result!r}"
# A braced-form unknown is kept verbatim too.
t2 = string.Template("$known $unknown")
s2 = t2.safe_substitute(known="hi")
assert "hi" in s2 and "$unknown" in s2, f"safe_substitute = {s2!r}"
print("template_safe_substitute_keeps_unknown OK")
"###);
    assert_output(&out, r###"template_safe_substitute_keeps_unknown OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/template_subclass_custom_pattern.py`.
#[test]
fn test_gen_behavior_std_libs_string_template_subclass_custom_pattern() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "template_subclass_custom_pattern"
# subject = "string.Template"
# kind = "semantic"
# xfail = "string.Template subclassing relies on the substitution engine that is a silent dict-stub on mamba (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Template: a Template subclass overriding `pattern` with a custom @@braced@@ group substitutes through the new grammar and safe_substitute keeps unresolved placeholders intact"""
import string


class MyTemplate(string.Template):
    pattern = r"""
        \$(?:
          (?P<escaped>\$)                    |
          (?P<named>[_a-z][_a-z0-9]*)        |
          @@(?P<braced>[_a-z][_a-z0-9]*)@@   |
          (?P<invalid>)                      |
        )
        """


tmpl = "PyCon in $@@location@@"
t = MyTemplate(tmpl)
_raised = False
try:
    t.substitute({})
except KeyError:
    _raised = True
assert _raised, "custom pattern missing key raises KeyError"
assert t.substitute({"location": "Cleveland"}) == "PyCon in Cleveland", "custom pattern substitute"
assert t.safe_substitute() == tmpl, "custom pattern safe_substitute keeps text"
assert t.safe_substitute({"location": "Cleveland"}) == "PyCon in Cleveland", "custom pattern safe_substitute"
print("template_subclass_custom_pattern OK")
"###);
    assert_output(&out, r###"template_subclass_custom_pattern OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/template_subclass_flags_case_sensitive.py`.
#[test]
fn test_gen_behavior_std_libs_string_template_subclass_flags_case_sensitive() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "template_subclass_flags_case_sensitive"
# subject = "string.Template"
# kind = "semantic"
# xfail = "string.Template subclassing relies on the substitution engine that is a silent dict-stub on mamba (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Template: a Template subclass setting `flags = 0` disables the default re.IGNORECASE so a mixed-case '$wHO' becomes an invalid placeholder (substitute raises ValueError, safe_substitute keeps it)"""
import string


class CaseSensitive(string.Template):
    flags = 0


s = CaseSensitive("$wHO likes ${WHAT} for ${meal}")
d = {"wHO": "tim", "WHAT": "ham", "meal": "dinner", "w": "fred"}
# '$wHO' no longer matches the lowercase idpattern -> invalid -> ValueError.
_raised = False
try:
    s.substitute(d)
except ValueError:
    _raised = True
assert _raised, "flags=0 makes '$wHO' invalid -> ValueError"
# safe_substitute keeps the invalid parts and fills what it can.
assert s.safe_substitute(d) == "fredHO likes ${WHAT} for dinner", "flags override safe"
print("template_subclass_flags_case_sensitive OK")
"###);
    assert_output(&out, r###"template_subclass_flags_case_sensitive OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/template_subclass_split_idpattern.py`.
#[test]
fn test_gen_behavior_std_libs_string_template_subclass_split_idpattern() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "template_subclass_split_idpattern"
# subject = "string.Template"
# kind = "semantic"
# xfail = "string.Template subclassing relies on the substitution engine that is a silent dict-stub on mamba (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Template: a Template subclass with separate idpattern (lower) and braceidpattern (upper) honors the split grammar: '$foo ${BAR}' resolves but '$FOO' and '${bar}' are invalid placeholders"""
import string


class SplitPattern(string.Template):
    idpattern = "[a-z]+"
    braceidpattern = "[A-Z]+"
    flags = 0


m = {"foo": "foo", "BAR": "BAR"}
assert SplitPattern("$foo ${BAR}").substitute(m) == "foo BAR", "split id/brace patterns"
# Unbraced upper-case and braced lower-case violate the split grammar.
for text in ("$FOO", "${bar}"):
    _raised = False
    try:
        SplitPattern(text).substitute(m)
    except ValueError:
        _raised = True
    assert _raised, f"expected ValueError for {text!r}"
print("template_subclass_split_idpattern OK")
"###);
    assert_output(&out, r###"template_subclass_split_idpattern OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/template_unicode_values.py`.
#[test]
fn test_gen_behavior_std_libs_string_template_unicode_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "template_unicode_values"
# subject = "string.Template"
# kind = "semantic"
# xfail = "string.Template is a silent dict-stub on mamba; .substitute() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.Template: substitution copies arbitrary unicode/control characters from the mapping verbatim into the result"""
import string

s = string.Template("$who likes $what")
d = {"who": "t\xffm", "what": "f\xfe\x0ced"}
assert s.substitute(d) == "t\xffm likes f\xfe\x0ced", "unicode/control values copied verbatim"
print("template_unicode_values OK")
"###);
    assert_output(&out, r###"template_unicode_values OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/test_template__test_braced_override.py`.
#[test]
fn test_gen_behavior_std_libs_string_test_template__test_braced_override() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "test_template__test_braced_override"
# subject = "cpython.test_string.TestTemplate.test_braced_override"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::TestTemplate::test_braced_override
"""Auto-ported test: TestTemplate::test_braced_override (CPython 3.12 oracle)."""


import unittest
import string
from string import Template


class Bag:
    pass

class Mapping:

    def __getitem__(self, name):
        obj = self
        for part in name.split('.'):
            try:
                obj = getattr(obj, part)
            except AttributeError:
                raise KeyError(name)
        return obj


# --- test body ---
class MyTemplate(Template):
    pattern = '\n            \\$(?:\n              (?P<escaped>$)                     |\n              (?P<named>[_a-z][_a-z0-9]*)        |\n              @@(?P<braced>[_a-z][_a-z0-9]*)@@   |\n              (?P<invalid>)                      |\n           )\n           '
tmpl = 'PyCon in $@@location@@'
t = MyTemplate(tmpl)

try:
    t.substitute({})
    raise AssertionError('expected KeyError')
except KeyError:
    pass
val = t.substitute({'location': 'Cleveland'})

assert val == 'PyCon in Cleveland'
print("TestTemplate::test_braced_override: ok")
"###);
    assert_output(&out, r###"TestTemplate::test_braced_override: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/test_template__test_braced_override_safe.py`.
#[test]
fn test_gen_behavior_std_libs_string_test_template__test_braced_override_safe() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "test_template__test_braced_override_safe"
# subject = "cpython.test_string.TestTemplate.test_braced_override_safe"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::TestTemplate::test_braced_override_safe
"""Auto-ported test: TestTemplate::test_braced_override_safe (CPython 3.12 oracle)."""


import unittest
import string
from string import Template


class Bag:
    pass

class Mapping:

    def __getitem__(self, name):
        obj = self
        for part in name.split('.'):
            try:
                obj = getattr(obj, part)
            except AttributeError:
                raise KeyError(name)
        return obj


# --- test body ---
class MyTemplate(Template):
    pattern = '\n            \\$(?:\n              (?P<escaped>$)                     |\n              (?P<named>[_a-z][_a-z0-9]*)        |\n              @@(?P<braced>[_a-z][_a-z0-9]*)@@   |\n              (?P<invalid>)                      |\n           )\n           '
tmpl = 'PyCon in $@@location@@'
t = MyTemplate(tmpl)

assert t.safe_substitute() == tmpl
val = t.safe_substitute({'location': 'Cleveland'})

assert val == 'PyCon in Cleveland'
print("TestTemplate::test_braced_override_safe: ok")
"###);
    assert_output(&out, r###"TestTemplate::test_braced_override_safe: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/test_template__test_flags_override.py`.
#[test]
fn test_gen_behavior_std_libs_string_test_template__test_flags_override() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "test_template__test_flags_override"
# subject = "cpython.test_string.TestTemplate.test_flags_override"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::TestTemplate::test_flags_override
"""Auto-ported test: TestTemplate::test_flags_override (CPython 3.12 oracle)."""


import unittest
import string
from string import Template


class Bag:
    pass

class Mapping:

    def __getitem__(self, name):
        obj = self
        for part in name.split('.'):
            try:
                obj = getattr(obj, part)
            except AttributeError:
                raise KeyError(name)
        return obj


# --- test body ---
class MyPattern(Template):
    flags = 0
s = MyPattern('$wHO likes ${WHAT} for ${meal}')
d = dict(wHO='tim', WHAT='ham', meal='dinner', w='fred')

try:
    s.substitute(d)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert s.safe_substitute(d) == 'fredHO likes ${WHAT} for dinner'
print("TestTemplate::test_flags_override: ok")
"###);
    assert_output(&out, r###"TestTemplate::test_flags_override: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/test_template__test_idpattern_override_inside_outside.py`.
#[test]
fn test_gen_behavior_std_libs_string_test_template__test_idpattern_override_inside_outside() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "test_template__test_idpattern_override_inside_outside"
# subject = "cpython.test_string.TestTemplate.test_idpattern_override_inside_outside"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::TestTemplate::test_idpattern_override_inside_outside
"""Auto-ported test: TestTemplate::test_idpattern_override_inside_outside (CPython 3.12 oracle)."""


import unittest
import string
from string import Template


class Bag:
    pass

class Mapping:

    def __getitem__(self, name):
        obj = self
        for part in name.split('.'):
            try:
                obj = getattr(obj, part)
            except AttributeError:
                raise KeyError(name)
        return obj


# --- test body ---
class MyPattern(Template):
    idpattern = '[a-z]+'
    braceidpattern = '[A-Z]+'
    flags = 0
m = dict(foo='foo', BAR='BAR')
s = MyPattern('$foo ${BAR}')

assert s.substitute(m) == 'foo BAR'
print("TestTemplate::test_idpattern_override_inside_outside: ok")
"###);
    assert_output(&out, r###"TestTemplate::test_idpattern_override_inside_outside: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/test_template__test_idpattern_override_inside_outside_invalid_unbraced.py`.
#[test]
fn test_gen_behavior_std_libs_string_test_template__test_idpattern_override_inside_outside_invalid_unbraced() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "test_template__test_idpattern_override_inside_outside_invalid_unbraced"
# subject = "cpython.test_string.TestTemplate.test_idpattern_override_inside_outside_invalid_unbraced"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::TestTemplate::test_idpattern_override_inside_outside_invalid_unbraced
"""Auto-ported test: TestTemplate::test_idpattern_override_inside_outside_invalid_unbraced (CPython 3.12 oracle)."""


import unittest
import string
from string import Template


class Bag:
    pass

class Mapping:

    def __getitem__(self, name):
        obj = self
        for part in name.split('.'):
            try:
                obj = getattr(obj, part)
            except AttributeError:
                raise KeyError(name)
        return obj


# --- test body ---
class MyPattern(Template):
    idpattern = '[a-z]+'
    braceidpattern = '[A-Z]+'
    flags = 0
m = dict(foo='foo', BAR='BAR')
s = MyPattern('$FOO')

try:
    s.substitute(m)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
s = MyPattern('${bar}')

try:
    s.substitute(m)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("TestTemplate::test_idpattern_override_inside_outside_invalid_unbraced: ok")
"###);
    assert_output(&out, r###"TestTemplate::test_idpattern_override_inside_outside_invalid_unbraced: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/test_template__test_invalid_with_no_lines.py`.
#[test]
fn test_gen_behavior_std_libs_string_test_template__test_invalid_with_no_lines() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "test_template__test_invalid_with_no_lines"
# subject = "cpython.test_string.TestTemplate.test_invalid_with_no_lines"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::TestTemplate::test_invalid_with_no_lines
"""Auto-ported test: TestTemplate::test_invalid_with_no_lines (CPython 3.12 oracle)."""


import unittest
import string
from string import Template


class Bag:
    pass

class Mapping:

    def __getitem__(self, name):
        obj = self
        for part in name.split('.'):
            try:
                obj = getattr(obj, part)
            except AttributeError:
                raise KeyError(name)
        return obj


# --- test body ---
class MyTemplate(Template):
    pattern = '\n              (?P<invalid>) |\n              unreachable(\n                (?P<named>)   |\n                (?P<braced>)  |\n                (?P<escaped>)\n              )\n            '
s = MyTemplate('')
try:
    s.substitute({})
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import types as _types_aR
    err = _types_aR.SimpleNamespace(exception=_aR_e)

assert 'line 1, col 1' in str(err.exception)
print("TestTemplate::test_invalid_with_no_lines: ok")
"###);
    assert_output(&out, r###"TestTemplate::test_invalid_with_no_lines: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/test_template__test_regular_templates_with_braces.py`.
#[test]
fn test_gen_behavior_std_libs_string_test_template__test_regular_templates_with_braces() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "test_template__test_regular_templates_with_braces"
# subject = "cpython.test_string.TestTemplate.test_regular_templates_with_braces"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::TestTemplate::test_regular_templates_with_braces
"""Auto-ported test: TestTemplate::test_regular_templates_with_braces (CPython 3.12 oracle)."""


import unittest
import string
from string import Template


class Bag:
    pass

class Mapping:

    def __getitem__(self, name):
        obj = self
        for part in name.split('.'):
            try:
                obj = getattr(obj, part)
            except AttributeError:
                raise KeyError(name)
        return obj


# --- test body ---
s = Template('$who likes ${what} for ${meal}')
d = dict(who='tim', what='ham', meal='dinner')

assert s.substitute(d) == 'tim likes ham for dinner'

try:
    s.substitute(dict(who='tim', what='ham'))
    raise AssertionError('expected KeyError')
except KeyError:
    pass
print("TestTemplate::test_regular_templates_with_braces: ok")
"###);
    assert_output(&out, r###"TestTemplate::test_regular_templates_with_braces: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/test_template__test_regular_templates_with_non_letters.py`.
#[test]
fn test_gen_behavior_std_libs_string_test_template__test_regular_templates_with_non_letters() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "test_template__test_regular_templates_with_non_letters"
# subject = "cpython.test_string.TestTemplate.test_regular_templates_with_non_letters"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::TestTemplate::test_regular_templates_with_non_letters
"""Auto-ported test: TestTemplate::test_regular_templates_with_non_letters (CPython 3.12 oracle)."""


import unittest
import string
from string import Template


class Bag:
    pass

class Mapping:

    def __getitem__(self, name):
        obj = self
        for part in name.split('.'):
            try:
                obj = getattr(obj, part)
            except AttributeError:
                raise KeyError(name)
        return obj


# --- test body ---
s = Template('$_wh0_ likes ${_w_h_a_t_} for ${mea1}')
d = dict(_wh0_='tim', _w_h_a_t_='ham', mea1='dinner')

assert s.substitute(d) == 'tim likes ham for dinner'
print("TestTemplate::test_regular_templates_with_non_letters: ok")
"###);
    assert_output(&out, r###"TestTemplate::test_regular_templates_with_non_letters: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/test_template__test_regular_templates_with_upper_case.py`.
#[test]
fn test_gen_behavior_std_libs_string_test_template__test_regular_templates_with_upper_case() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "test_template__test_regular_templates_with_upper_case"
# subject = "cpython.test_string.TestTemplate.test_regular_templates_with_upper_case"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::TestTemplate::test_regular_templates_with_upper_case
"""Auto-ported test: TestTemplate::test_regular_templates_with_upper_case (CPython 3.12 oracle)."""


import unittest
import string
from string import Template


class Bag:
    pass

class Mapping:

    def __getitem__(self, name):
        obj = self
        for part in name.split('.'):
            try:
                obj = getattr(obj, part)
            except AttributeError:
                raise KeyError(name)
        return obj


# --- test body ---
s = Template('$WHO likes ${WHAT} for ${MEAL}')
d = dict(WHO='tim', WHAT='ham', MEAL='dinner')

assert s.substitute(d) == 'tim likes ham for dinner'
print("TestTemplate::test_regular_templates_with_upper_case: ok")
"###);
    assert_output(&out, r###"TestTemplate::test_regular_templates_with_upper_case: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/string/test_template__test_unicode_values.py`.
#[test]
fn test_gen_behavior_std_libs_string_test_template__test_unicode_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "test_template__test_unicode_values"
# subject = "cpython.test_string.TestTemplate.test_unicode_values"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::TestTemplate::test_unicode_values
"""Auto-ported test: TestTemplate::test_unicode_values (CPython 3.12 oracle)."""


import unittest
import string
from string import Template


class Bag:
    pass

class Mapping:

    def __getitem__(self, name):
        obj = self
        for part in name.split('.'):
            try:
                obj = getattr(obj, part)
            except AttributeError:
                raise KeyError(name)
        return obj


# --- test body ---
s = Template('$who likes $what')
d = dict(who='tÿm', what='fþ\x0ced')

assert s.substitute(d) == 'tÿm likes fþ\x0ced'
print("TestTemplate::test_unicode_values: ok")
"###);
    assert_output(&out, r###"TestTemplate::test_unicode_values: ok
"###);
}
