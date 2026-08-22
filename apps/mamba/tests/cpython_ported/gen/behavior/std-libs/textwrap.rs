use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/textwrap/dedent_declining_indent_smallest_wins.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_dedent_declining_indent_smallest_wins() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "dedent_declining_indent_smallest_wins"
# subject = "textwrap.dedent"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.dedent: with declining indentation the smallest content prefix wins; blank/whitespace-only interior lines do not change the result"""
import textwrap

# The smallest content prefix (here one space) is the common prefix.
declining = "     Foo\n    Bar\n"
assert textwrap.dedent(declining) == " Foo\nBar\n", (
    f"declining = {textwrap.dedent(declining)!r}"
)
# Blank or whitespace-only interior lines do not change that result.
assert textwrap.dedent("     Foo\n\n    Bar\n") == " Foo\n\nBar\n", "blank interior"
assert textwrap.dedent("     Foo\n    \n    Bar\n") == " Foo\n\nBar\n", "ws interior"
print("dedent_declining_indent_smallest_wins OK")
"###);
    assert_output(&out, r###"dedent_declining_indent_smallest_wins OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/dedent_ignores_whitespace_only_lines.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_dedent_ignores_whitespace_only_lines() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "dedent_ignores_whitespace_only_lines"
# subject = "textwrap.dedent"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.dedent: a whitespace-only line does not count toward the common prefix and is normalized to empty in the output"""
import textwrap

# The "  \n" line between content lines is whitespace-only: it is ignored when
# computing the common prefix and blanked to "\n" in the output.
ws_only = "  Hello there.\n  \n  How are ya?\n  Oh good.\n"
assert textwrap.dedent(ws_only) == "Hello there.\n\nHow are ya?\nOh good.\n", (
    f"ws_only = {textwrap.dedent(ws_only)!r}"
)
print("dedent_ignores_whitespace_only_lines OK")
"###);
    assert_output(&out, r###"dedent_ignores_whitespace_only_lines OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/dedent_keeps_internal_tabs_and_is_idempotent.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_dedent_keeps_internal_tabs_and_is_idempotent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "dedent_keeps_internal_tabs_and_is_idempotent"
# subject = "textwrap.dedent"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.dedent: internal (non-leading) tabs are preserved verbatim and dedent is idempotent on already-dedented text"""
import textwrap

tabbed = "  hello\tthere\n  how are\tyou?"
once = textwrap.dedent(tabbed)
assert once == "hello\tthere\nhow are\tyou?", f"tabbed = {once!r}"
assert textwrap.dedent(once) == once, "dedent idempotent on dedented text"
print("dedent_keeps_internal_tabs_and_is_idempotent OK")
"###);
    assert_output(&out, r###"dedent_keeps_internal_tabs_and_is_idempotent OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/dedent_preserves_relative_indent.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_dedent_preserves_relative_indent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "dedent_preserves_relative_indent"
# subject = "textwrap.dedent"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.dedent: uneven (code-block) indentation keeps relative indentation after the common prefix is removed"""
import textwrap

code = "        def foo():\n            while 1:\n                return foo\n        "
assert textwrap.dedent(code) == "def foo():\n    while 1:\n        return foo\n", (
    f"code = {textwrap.dedent(code)!r}"
)
nested = "  Foo\n    Bar\n \n   Baz\n"
assert textwrap.dedent(nested) == "Foo\n  Bar\n\n Baz\n", (
    f"nested = {textwrap.dedent(nested)!r}"
)
print("dedent_preserves_relative_indent OK")
"###);
    assert_output(&out, r###"dedent_preserves_relative_indent OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/dedent_removes_common_prefix.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_dedent_removes_common_prefix() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "dedent_removes_common_prefix"
# subject = "textwrap.dedent"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.dedent: dedent strips the minimum common leading whitespace shared by all content lines"""
import textwrap

# A constant prefix is stripped from every content line.
even = "  Hello there.\n  How are ya?\n  Oh good."
assert textwrap.dedent(even) == "Hello there.\nHow are ya?\nOh good.", (
    f"even = {textwrap.dedent(even)!r}"
)
# The minimum (here 3 spaces) is removed; deeper lines keep the excess.
text = "   line1\n   line2\n     line3\n"
assert textwrap.dedent(text) == "line1\nline2\n  line3\n", (
    f"dedent result = {textwrap.dedent(text)!r}"
)
print("dedent_removes_common_prefix OK")
"###);
    assert_output(&out, r###"dedent_removes_common_prefix OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/dedent_test_case__test_dedent_declining.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_dedent_test_case__test_dedent_declining() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "dedent_test_case__test_dedent_declining"
# subject = "cpython.test_textwrap.DedentTestCase.test_dedent_declining"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::DedentTestCase::test_dedent_declining
"""Auto-ported test: DedentTestCase::test_dedent_declining (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


class BaseTestCase(unittest.TestCase):
    """Parent class with utility methods for textwrap tests."""

    def show(self, textin):
        if isinstance(textin, list):
            result = []
            for i in range(len(textin)):
                result.append('  %d: %r' % (i, textin[i]))
            result = '\n'.join(result) if result else '  no lines'
        elif isinstance(textin, str):
            result = '  %s\n' % repr(textin)
        return result

    def check(self, result, expect):
        self.assertEqual(result, expect, 'expected:\n%s\nbut got:\n%s' % (self.show(expect), self.show(result)))

    def check_wrap(self, text, width, expect, **kwargs):
        result = wrap(text, width, **kwargs)
        self.check(result, expect)

    def check_split(self, text, expect):
        result = self.wrapper._split(text)
        self.assertEqual(result, expect, '\nexpected %r\nbut got  %r' % (expect, result))


# --- test body ---
text = '     Foo\n    Bar\n'
expect = ' Foo\nBar\n'

assert expect == dedent(text)
text = '     Foo\n\n    Bar\n'
expect = ' Foo\n\nBar\n'

assert expect == dedent(text)
text = '     Foo\n    \n    Bar\n'
expect = ' Foo\n\nBar\n'

assert expect == dedent(text)
print("DedentTestCase::test_dedent_declining: ok")
"###);
    assert_output(&out, r###"DedentTestCase::test_dedent_declining: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/dedent_test_case__test_dedent_even.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_dedent_test_case__test_dedent_even() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "dedent_test_case__test_dedent_even"
# subject = "cpython.test_textwrap.DedentTestCase.test_dedent_even"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::DedentTestCase::test_dedent_even
"""Auto-ported test: DedentTestCase::test_dedent_even (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


class BaseTestCase(unittest.TestCase):
    """Parent class with utility methods for textwrap tests."""

    def show(self, textin):
        if isinstance(textin, list):
            result = []
            for i in range(len(textin)):
                result.append('  %d: %r' % (i, textin[i]))
            result = '\n'.join(result) if result else '  no lines'
        elif isinstance(textin, str):
            result = '  %s\n' % repr(textin)
        return result

    def check(self, result, expect):
        self.assertEqual(result, expect, 'expected:\n%s\nbut got:\n%s' % (self.show(expect), self.show(result)))

    def check_wrap(self, text, width, expect, **kwargs):
        result = wrap(text, width, **kwargs)
        self.check(result, expect)

    def check_split(self, text, expect):
        result = self.wrapper._split(text)
        self.assertEqual(result, expect, '\nexpected %r\nbut got  %r' % (expect, result))


# --- test body ---
text = '  Hello there.\n  How are ya?\n  Oh good.'
expect = 'Hello there.\nHow are ya?\nOh good.'

assert expect == dedent(text)
text = '  Hello there.\n\n  How are ya?\n  Oh good.\n'
expect = 'Hello there.\n\nHow are ya?\nOh good.\n'

assert expect == dedent(text)
text = '  Hello there.\n  \n  How are ya?\n  Oh good.\n'
expect = 'Hello there.\n\nHow are ya?\nOh good.\n'

assert expect == dedent(text)
print("DedentTestCase::test_dedent_even: ok")
"###);
    assert_output(&out, r###"DedentTestCase::test_dedent_even: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/dedent_test_case__test_dedent_nomargin.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_dedent_test_case__test_dedent_nomargin() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "dedent_test_case__test_dedent_nomargin"
# subject = "cpython.test_textwrap.DedentTestCase.test_dedent_nomargin"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::DedentTestCase::test_dedent_nomargin
"""Auto-ported test: DedentTestCase::test_dedent_nomargin (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def assertUnchanged(text):
    """assert that dedent() has no effect on 'text'"""

    assert text == dedent(text)
text = "Hello there.\nHow are you?\nOh good, I'm glad."
assertUnchanged(text)
text = 'Hello there.\n\nBoo!'
assertUnchanged(text)
text = 'Hello there.\n  This is indented.'
assertUnchanged(text)
text = 'Hello there.\n\n  Boo!\n'
assertUnchanged(text)
print("DedentTestCase::test_dedent_nomargin: ok")
"###);
    assert_output(&out, r###"DedentTestCase::test_dedent_nomargin: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/dedent_test_case__test_dedent_preserve_internal_tabs.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_dedent_test_case__test_dedent_preserve_internal_tabs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "dedent_test_case__test_dedent_preserve_internal_tabs"
# subject = "cpython.test_textwrap.DedentTestCase.test_dedent_preserve_internal_tabs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::DedentTestCase::test_dedent_preserve_internal_tabs
"""Auto-ported test: DedentTestCase::test_dedent_preserve_internal_tabs (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


class BaseTestCase(unittest.TestCase):
    """Parent class with utility methods for textwrap tests."""

    def show(self, textin):
        if isinstance(textin, list):
            result = []
            for i in range(len(textin)):
                result.append('  %d: %r' % (i, textin[i]))
            result = '\n'.join(result) if result else '  no lines'
        elif isinstance(textin, str):
            result = '  %s\n' % repr(textin)
        return result

    def check(self, result, expect):
        self.assertEqual(result, expect, 'expected:\n%s\nbut got:\n%s' % (self.show(expect), self.show(result)))

    def check_wrap(self, text, width, expect, **kwargs):
        result = wrap(text, width, **kwargs)
        self.check(result, expect)

    def check_split(self, text, expect):
        result = self.wrapper._split(text)
        self.assertEqual(result, expect, '\nexpected %r\nbut got  %r' % (expect, result))


# --- test body ---
text = '  hello\tthere\n  how are\tyou?'
expect = 'hello\tthere\nhow are\tyou?'

assert expect == dedent(text)

assert expect == dedent(expect)
print("DedentTestCase::test_dedent_preserve_internal_tabs: ok")
"###);
    assert_output(&out, r###"DedentTestCase::test_dedent_preserve_internal_tabs: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/dedent_test_case__test_dedent_preserve_margin_tabs.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_dedent_test_case__test_dedent_preserve_margin_tabs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "dedent_test_case__test_dedent_preserve_margin_tabs"
# subject = "cpython.test_textwrap.DedentTestCase.test_dedent_preserve_margin_tabs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::DedentTestCase::test_dedent_preserve_margin_tabs
"""Auto-ported test: DedentTestCase::test_dedent_preserve_margin_tabs (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def assertUnchanged(text):
    """assert that dedent() has no effect on 'text'"""

    assert text == dedent(text)
text = '  hello there\n\thow are you?'
assertUnchanged(text)
text = '        hello there\n\thow are you?'
assertUnchanged(text)
text = '\thello there\n\thow are you?'
expect = 'hello there\nhow are you?'

assert expect == dedent(text)
text = '  \thello there\n  \thow are you?'

assert expect == dedent(text)
text = '  \t  hello there\n  \t  how are you?'

assert expect == dedent(text)
text = '  \thello there\n  \t  how are you?'
expect = 'hello there\n  how are you?'

assert expect == dedent(text)
text = "  \thello there\n   \thow are you?\n \tI'm fine, thanks"
expect = " \thello there\n  \thow are you?\n\tI'm fine, thanks"

assert expect == dedent(text)
print("DedentTestCase::test_dedent_preserve_margin_tabs: ok")
"###);
    assert_output(&out, r###"DedentTestCase::test_dedent_preserve_margin_tabs: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/dedent_test_case__test_dedent_uneven.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_dedent_test_case__test_dedent_uneven() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "dedent_test_case__test_dedent_uneven"
# subject = "cpython.test_textwrap.DedentTestCase.test_dedent_uneven"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::DedentTestCase::test_dedent_uneven
"""Auto-ported test: DedentTestCase::test_dedent_uneven (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


class BaseTestCase(unittest.TestCase):
    """Parent class with utility methods for textwrap tests."""

    def show(self, textin):
        if isinstance(textin, list):
            result = []
            for i in range(len(textin)):
                result.append('  %d: %r' % (i, textin[i]))
            result = '\n'.join(result) if result else '  no lines'
        elif isinstance(textin, str):
            result = '  %s\n' % repr(textin)
        return result

    def check(self, result, expect):
        self.assertEqual(result, expect, 'expected:\n%s\nbut got:\n%s' % (self.show(expect), self.show(result)))

    def check_wrap(self, text, width, expect, **kwargs):
        result = wrap(text, width, **kwargs)
        self.check(result, expect)

    def check_split(self, text, expect):
        result = self.wrapper._split(text)
        self.assertEqual(result, expect, '\nexpected %r\nbut got  %r' % (expect, result))


# --- test body ---
text = '        def foo():\n            while 1:\n                return foo\n        '
expect = 'def foo():\n    while 1:\n        return foo\n'

assert expect == dedent(text)
text = '  Foo\n    Bar\n\n   Baz\n'
expect = 'Foo\n  Bar\n\n Baz\n'

assert expect == dedent(text)
text = '  Foo\n    Bar\n \n   Baz\n'
expect = 'Foo\n  Bar\n\n Baz\n'

assert expect == dedent(text)
print("DedentTestCase::test_dedent_uneven: ok")
"###);
    assert_output(&out, r###"DedentTestCase::test_dedent_uneven: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/fill_joins_lines_within_width.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_fill_joins_lines_within_width() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "fill_joins_lines_within_width"
# subject = "textwrap.fill"
# kind = "semantic"
# xfail = "textwrap.fill is a silent stub under mamba — returns the input unchanged, no wrap (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.fill: fill returns a single newline-joined string whose every line is within width"""
import textwrap

filled = textwrap.fill("one two three four five", width=12)
assert isinstance(filled, str), f"fill type = {type(filled)!r}"
lines = filled.split("\n")
assert len(lines) > 1, f"expected multiple lines, got {lines!r}"
assert all(len(line) <= 12 for line in lines), f"fill lines <= 12: {lines!r}"
print("fill_joins_lines_within_width OK")
"###);
    assert_output(&out, r###"fill_joins_lines_within_width OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/fill_short_text_no_wrap.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_fill_short_text_no_wrap() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "fill_short_text_no_wrap"
# subject = "textwrap.fill"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.fill: fill on text shorter than width returns it unchanged with no newline; fill on the empty string returns the empty string"""
import textwrap

assert textwrap.fill("one two three", width=100) == "one two three", "short text no wrap"
assert textwrap.fill("") == "", "fill empty = ''"
print("fill_short_text_no_wrap OK")
"###);
    assert_output(&out, r###"fill_short_text_no_wrap OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/indent_predicate_selects_lines.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_indent_predicate_selects_lines() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "indent_predicate_selects_lines"
# subject = "textwrap.indent"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.indent: indent with a predicate= callable prefixes only the lines for which the predicate returns True"""
import textwrap

out = textwrap.indent(
    "line1\nline2\nline3", "* ", predicate=lambda s: s.startswith("line2")
)
assert out == "line1\n* line2\nline3", f"indent predicate = {out!r}"
print("indent_predicate_selects_lines OK")
"###);
    assert_output(&out, r###"indent_predicate_selects_lines OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/indent_prefixes_non_empty_lines.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_indent_prefixes_non_empty_lines() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "indent_prefixes_non_empty_lines"
# subject = "textwrap.indent"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.indent: indent prepends the prefix to every non-empty line by default, leaving empty lines untouched"""
import textwrap

# Default predicate prefixes non-empty lines only; the blank line is untouched.
out = textwrap.indent("line1\nline2\n\nline3", "# ")
assert out == "# line1\n# line2\n\n# line3", f"indent = {out!r}"
# Simple multi-line prefix with a trailing newline preserved.
out2 = textwrap.indent("line1\nline2\nline3", ">> ")
assert out2 == ">> line1\n>> line2\n>> line3", f"indent = {out2!r}"
print("indent_prefixes_non_empty_lines OK")
"###);
    assert_output(&out, r###"indent_prefixes_non_empty_lines OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/indent_test_case__test_indent_all_lines.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_indent_test_case__test_indent_all_lines() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "indent_test_case__test_indent_all_lines"
# subject = "cpython.test_textwrap.IndentTestCase.test_indent_all_lines"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::IndentTestCase::test_indent_all_lines
"""Auto-ported test: IndentTestCase::test_indent_all_lines (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


class BaseTestCase(unittest.TestCase):
    """Parent class with utility methods for textwrap tests."""

    def show(self, textin):
        if isinstance(textin, list):
            result = []
            for i in range(len(textin)):
                result.append('  %d: %r' % (i, textin[i]))
            result = '\n'.join(result) if result else '  no lines'
        elif isinstance(textin, str):
            result = '  %s\n' % repr(textin)
        return result

    def check(self, result, expect):
        self.assertEqual(result, expect, 'expected:\n%s\nbut got:\n%s' % (self.show(expect), self.show(result)))

    def check_wrap(self, text, width, expect, **kwargs):
        result = wrap(text, width, **kwargs)
        self.check(result, expect)

    def check_split(self, text, expect):
        result = self.wrapper._split(text)
        self.assertEqual(result, expect, '\nexpected %r\nbut got  %r' % (expect, result))


# --- test body ---
ROUNDTRIP_CASES = ('Hi.\nThis is a test.\nTesting.', 'Hi.\nThis is a test.\n\nTesting.', '\nHi.\nThis is a test.\nTesting.\n')
CASES = ROUNDTRIP_CASES + ('Hi.\r\nThis is a test.\r\nTesting.\r\n', '\nHi.\r\nThis is a test.\n\r\nTesting.\r\n\n')
prefix = '  '
expected = ('  Hi.\n  This is a test.\n  Testing.', '  Hi.\n  This is a test.\n  \n  Testing.', '  \n  Hi.\n  This is a test.\n  Testing.\n', '  Hi.\r\n  This is a test.\r\n  Testing.\r\n', '  \n  Hi.\r\n  This is a test.\n  \r\n  Testing.\r\n  \n')
predicate = lambda line: True
for text, expect in zip(CASES, expected):

    assert indent(text, prefix, predicate) == expect
print("IndentTestCase::test_indent_all_lines: ok")
"###);
    assert_output(&out, r###"IndentTestCase::test_indent_all_lines: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/indent_test_case__test_indent_default.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_indent_test_case__test_indent_default() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "indent_test_case__test_indent_default"
# subject = "cpython.test_textwrap.IndentTestCase.test_indent_default"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::IndentTestCase::test_indent_default
"""Auto-ported test: IndentTestCase::test_indent_default (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


class BaseTestCase(unittest.TestCase):
    """Parent class with utility methods for textwrap tests."""

    def show(self, textin):
        if isinstance(textin, list):
            result = []
            for i in range(len(textin)):
                result.append('  %d: %r' % (i, textin[i]))
            result = '\n'.join(result) if result else '  no lines'
        elif isinstance(textin, str):
            result = '  %s\n' % repr(textin)
        return result

    def check(self, result, expect):
        self.assertEqual(result, expect, 'expected:\n%s\nbut got:\n%s' % (self.show(expect), self.show(result)))

    def check_wrap(self, text, width, expect, **kwargs):
        result = wrap(text, width, **kwargs)
        self.check(result, expect)

    def check_split(self, text, expect):
        result = self.wrapper._split(text)
        self.assertEqual(result, expect, '\nexpected %r\nbut got  %r' % (expect, result))


# --- test body ---
ROUNDTRIP_CASES = ('Hi.\nThis is a test.\nTesting.', 'Hi.\nThis is a test.\n\nTesting.', '\nHi.\nThis is a test.\nTesting.\n')
CASES = ROUNDTRIP_CASES + ('Hi.\r\nThis is a test.\r\nTesting.\r\n', '\nHi.\r\nThis is a test.\n\r\nTesting.\r\n\n')
prefix = '  '
expected = ('  Hi.\n  This is a test.\n  Testing.', '  Hi.\n  This is a test.\n\n  Testing.', '\n  Hi.\n  This is a test.\n  Testing.\n', '  Hi.\r\n  This is a test.\r\n  Testing.\r\n', '\n  Hi.\r\n  This is a test.\n\r\n  Testing.\r\n\n')
for text, expect in zip(CASES, expected):

    assert indent(text, prefix) == expect
print("IndentTestCase::test_indent_default: ok")
"###);
    assert_output(&out, r###"IndentTestCase::test_indent_default: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/indent_test_case__test_indent_empty_lines.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_indent_test_case__test_indent_empty_lines() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "indent_test_case__test_indent_empty_lines"
# subject = "cpython.test_textwrap.IndentTestCase.test_indent_empty_lines"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::IndentTestCase::test_indent_empty_lines
"""Auto-ported test: IndentTestCase::test_indent_empty_lines (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


class BaseTestCase(unittest.TestCase):
    """Parent class with utility methods for textwrap tests."""

    def show(self, textin):
        if isinstance(textin, list):
            result = []
            for i in range(len(textin)):
                result.append('  %d: %r' % (i, textin[i]))
            result = '\n'.join(result) if result else '  no lines'
        elif isinstance(textin, str):
            result = '  %s\n' % repr(textin)
        return result

    def check(self, result, expect):
        self.assertEqual(result, expect, 'expected:\n%s\nbut got:\n%s' % (self.show(expect), self.show(result)))

    def check_wrap(self, text, width, expect, **kwargs):
        result = wrap(text, width, **kwargs)
        self.check(result, expect)

    def check_split(self, text, expect):
        result = self.wrapper._split(text)
        self.assertEqual(result, expect, '\nexpected %r\nbut got  %r' % (expect, result))


# --- test body ---
ROUNDTRIP_CASES = ('Hi.\nThis is a test.\nTesting.', 'Hi.\nThis is a test.\n\nTesting.', '\nHi.\nThis is a test.\nTesting.\n')
CASES = ROUNDTRIP_CASES + ('Hi.\r\nThis is a test.\r\nTesting.\r\n', '\nHi.\r\nThis is a test.\n\r\nTesting.\r\n\n')
prefix = '  '
expected = ('Hi.\nThis is a test.\nTesting.', 'Hi.\nThis is a test.\n  \nTesting.', '  \nHi.\nThis is a test.\nTesting.\n', 'Hi.\r\nThis is a test.\r\nTesting.\r\n', '  \nHi.\r\nThis is a test.\n  \r\nTesting.\r\n  \n')
predicate = lambda line: not line.strip()
for text, expect in zip(CASES, expected):

    assert indent(text, prefix, predicate) == expect
print("IndentTestCase::test_indent_empty_lines: ok")
"###);
    assert_output(&out, r###"IndentTestCase::test_indent_empty_lines: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/indent_test_case__test_indent_explicit_default.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_indent_test_case__test_indent_explicit_default() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "indent_test_case__test_indent_explicit_default"
# subject = "cpython.test_textwrap.IndentTestCase.test_indent_explicit_default"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::IndentTestCase::test_indent_explicit_default
"""Auto-ported test: IndentTestCase::test_indent_explicit_default (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


class BaseTestCase(unittest.TestCase):
    """Parent class with utility methods for textwrap tests."""

    def show(self, textin):
        if isinstance(textin, list):
            result = []
            for i in range(len(textin)):
                result.append('  %d: %r' % (i, textin[i]))
            result = '\n'.join(result) if result else '  no lines'
        elif isinstance(textin, str):
            result = '  %s\n' % repr(textin)
        return result

    def check(self, result, expect):
        self.assertEqual(result, expect, 'expected:\n%s\nbut got:\n%s' % (self.show(expect), self.show(result)))

    def check_wrap(self, text, width, expect, **kwargs):
        result = wrap(text, width, **kwargs)
        self.check(result, expect)

    def check_split(self, text, expect):
        result = self.wrapper._split(text)
        self.assertEqual(result, expect, '\nexpected %r\nbut got  %r' % (expect, result))


# --- test body ---
ROUNDTRIP_CASES = ('Hi.\nThis is a test.\nTesting.', 'Hi.\nThis is a test.\n\nTesting.', '\nHi.\nThis is a test.\nTesting.\n')
CASES = ROUNDTRIP_CASES + ('Hi.\r\nThis is a test.\r\nTesting.\r\n', '\nHi.\r\nThis is a test.\n\r\nTesting.\r\n\n')
prefix = '  '
expected = ('  Hi.\n  This is a test.\n  Testing.', '  Hi.\n  This is a test.\n\n  Testing.', '\n  Hi.\n  This is a test.\n  Testing.\n', '  Hi.\r\n  This is a test.\r\n  Testing.\r\n', '\n  Hi.\r\n  This is a test.\n\r\n  Testing.\r\n\n')
for text, expect in zip(CASES, expected):

    assert indent(text, prefix, None) == expect
print("IndentTestCase::test_indent_explicit_default: ok")
"###);
    assert_output(&out, r###"IndentTestCase::test_indent_explicit_default: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/indent_test_case__test_indent_no_lines.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_indent_test_case__test_indent_no_lines() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "indent_test_case__test_indent_no_lines"
# subject = "cpython.test_textwrap.IndentTestCase.test_indent_no_lines"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::IndentTestCase::test_indent_no_lines
"""Auto-ported test: IndentTestCase::test_indent_no_lines (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


class BaseTestCase(unittest.TestCase):
    """Parent class with utility methods for textwrap tests."""

    def show(self, textin):
        if isinstance(textin, list):
            result = []
            for i in range(len(textin)):
                result.append('  %d: %r' % (i, textin[i]))
            result = '\n'.join(result) if result else '  no lines'
        elif isinstance(textin, str):
            result = '  %s\n' % repr(textin)
        return result

    def check(self, result, expect):
        self.assertEqual(result, expect, 'expected:\n%s\nbut got:\n%s' % (self.show(expect), self.show(result)))

    def check_wrap(self, text, width, expect, **kwargs):
        result = wrap(text, width, **kwargs)
        self.check(result, expect)

    def check_split(self, text, expect):
        result = self.wrapper._split(text)
        self.assertEqual(result, expect, '\nexpected %r\nbut got  %r' % (expect, result))


# --- test body ---
ROUNDTRIP_CASES = ('Hi.\nThis is a test.\nTesting.', 'Hi.\nThis is a test.\n\nTesting.', '\nHi.\nThis is a test.\nTesting.\n')
CASES = ROUNDTRIP_CASES + ('Hi.\r\nThis is a test.\r\nTesting.\r\n', '\nHi.\r\nThis is a test.\n\r\nTesting.\r\n\n')
predicate = lambda line: False
for text in CASES:

    assert indent(text, '    ', predicate) == text
print("IndentTestCase::test_indent_no_lines: ok")
"###);
    assert_output(&out, r###"IndentTestCase::test_indent_no_lines: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/indent_test_case__test_indent_nomargin_all_lines.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_indent_test_case__test_indent_nomargin_all_lines() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "indent_test_case__test_indent_nomargin_all_lines"
# subject = "cpython.test_textwrap.IndentTestCase.test_indent_nomargin_all_lines"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::IndentTestCase::test_indent_nomargin_all_lines
"""Auto-ported test: IndentTestCase::test_indent_nomargin_all_lines (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


class BaseTestCase(unittest.TestCase):
    """Parent class with utility methods for textwrap tests."""

    def show(self, textin):
        if isinstance(textin, list):
            result = []
            for i in range(len(textin)):
                result.append('  %d: %r' % (i, textin[i]))
            result = '\n'.join(result) if result else '  no lines'
        elif isinstance(textin, str):
            result = '  %s\n' % repr(textin)
        return result

    def check(self, result, expect):
        self.assertEqual(result, expect, 'expected:\n%s\nbut got:\n%s' % (self.show(expect), self.show(result)))

    def check_wrap(self, text, width, expect, **kwargs):
        result = wrap(text, width, **kwargs)
        self.check(result, expect)

    def check_split(self, text, expect):
        result = self.wrapper._split(text)
        self.assertEqual(result, expect, '\nexpected %r\nbut got  %r' % (expect, result))


# --- test body ---
ROUNDTRIP_CASES = ('Hi.\nThis is a test.\nTesting.', 'Hi.\nThis is a test.\n\nTesting.', '\nHi.\nThis is a test.\nTesting.\n')
CASES = ROUNDTRIP_CASES + ('Hi.\r\nThis is a test.\r\nTesting.\r\n', '\nHi.\r\nThis is a test.\n\r\nTesting.\r\n\n')
predicate = lambda line: True
for text in CASES:

    assert indent(text, '', predicate) == text
print("IndentTestCase::test_indent_nomargin_all_lines: ok")
"###);
    assert_output(&out, r###"IndentTestCase::test_indent_nomargin_all_lines: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/indent_test_case__test_indent_nomargin_default.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_indent_test_case__test_indent_nomargin_default() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "indent_test_case__test_indent_nomargin_default"
# subject = "cpython.test_textwrap.IndentTestCase.test_indent_nomargin_default"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::IndentTestCase::test_indent_nomargin_default
"""Auto-ported test: IndentTestCase::test_indent_nomargin_default (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


class BaseTestCase(unittest.TestCase):
    """Parent class with utility methods for textwrap tests."""

    def show(self, textin):
        if isinstance(textin, list):
            result = []
            for i in range(len(textin)):
                result.append('  %d: %r' % (i, textin[i]))
            result = '\n'.join(result) if result else '  no lines'
        elif isinstance(textin, str):
            result = '  %s\n' % repr(textin)
        return result

    def check(self, result, expect):
        self.assertEqual(result, expect, 'expected:\n%s\nbut got:\n%s' % (self.show(expect), self.show(result)))

    def check_wrap(self, text, width, expect, **kwargs):
        result = wrap(text, width, **kwargs)
        self.check(result, expect)

    def check_split(self, text, expect):
        result = self.wrapper._split(text)
        self.assertEqual(result, expect, '\nexpected %r\nbut got  %r' % (expect, result))


# --- test body ---
ROUNDTRIP_CASES = ('Hi.\nThis is a test.\nTesting.', 'Hi.\nThis is a test.\n\nTesting.', '\nHi.\nThis is a test.\nTesting.\n')
CASES = ROUNDTRIP_CASES + ('Hi.\r\nThis is a test.\r\nTesting.\r\n', '\nHi.\r\nThis is a test.\n\r\nTesting.\r\n\n')
for text in CASES:

    assert indent(text, '') == text
print("IndentTestCase::test_indent_nomargin_default: ok")
"###);
    assert_output(&out, r###"IndentTestCase::test_indent_nomargin_default: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/indent_test_case__test_indent_nomargin_explicit_default.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_indent_test_case__test_indent_nomargin_explicit_default() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "indent_test_case__test_indent_nomargin_explicit_default"
# subject = "cpython.test_textwrap.IndentTestCase.test_indent_nomargin_explicit_default"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::IndentTestCase::test_indent_nomargin_explicit_default
"""Auto-ported test: IndentTestCase::test_indent_nomargin_explicit_default (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


class BaseTestCase(unittest.TestCase):
    """Parent class with utility methods for textwrap tests."""

    def show(self, textin):
        if isinstance(textin, list):
            result = []
            for i in range(len(textin)):
                result.append('  %d: %r' % (i, textin[i]))
            result = '\n'.join(result) if result else '  no lines'
        elif isinstance(textin, str):
            result = '  %s\n' % repr(textin)
        return result

    def check(self, result, expect):
        self.assertEqual(result, expect, 'expected:\n%s\nbut got:\n%s' % (self.show(expect), self.show(result)))

    def check_wrap(self, text, width, expect, **kwargs):
        result = wrap(text, width, **kwargs)
        self.check(result, expect)

    def check_split(self, text, expect):
        result = self.wrapper._split(text)
        self.assertEqual(result, expect, '\nexpected %r\nbut got  %r' % (expect, result))


# --- test body ---
ROUNDTRIP_CASES = ('Hi.\nThis is a test.\nTesting.', 'Hi.\nThis is a test.\n\nTesting.', '\nHi.\nThis is a test.\nTesting.\n')
CASES = ROUNDTRIP_CASES + ('Hi.\r\nThis is a test.\r\nTesting.\r\n', '\nHi.\r\nThis is a test.\n\r\nTesting.\r\n\n')
for text in CASES:

    assert indent(text, '', None) == text
print("IndentTestCase::test_indent_nomargin_explicit_default: ok")
"###);
    assert_output(&out, r###"IndentTestCase::test_indent_nomargin_explicit_default: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/indent_test_case__test_roundtrip_mixed.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_indent_test_case__test_roundtrip_mixed() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "indent_test_case__test_roundtrip_mixed"
# subject = "cpython.test_textwrap.IndentTestCase.test_roundtrip_mixed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::IndentTestCase::test_roundtrip_mixed
"""Auto-ported test: IndentTestCase::test_roundtrip_mixed (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


class BaseTestCase(unittest.TestCase):
    """Parent class with utility methods for textwrap tests."""

    def show(self, textin):
        if isinstance(textin, list):
            result = []
            for i in range(len(textin)):
                result.append('  %d: %r' % (i, textin[i]))
            result = '\n'.join(result) if result else '  no lines'
        elif isinstance(textin, str):
            result = '  %s\n' % repr(textin)
        return result

    def check(self, result, expect):
        self.assertEqual(result, expect, 'expected:\n%s\nbut got:\n%s' % (self.show(expect), self.show(result)))

    def check_wrap(self, text, width, expect, **kwargs):
        result = wrap(text, width, **kwargs)
        self.check(result, expect)

    def check_split(self, text, expect):
        result = self.wrapper._split(text)
        self.assertEqual(result, expect, '\nexpected %r\nbut got  %r' % (expect, result))


# --- test body ---
ROUNDTRIP_CASES = ('Hi.\nThis is a test.\nTesting.', 'Hi.\nThis is a test.\n\nTesting.', '\nHi.\nThis is a test.\nTesting.\n')
CASES = ROUNDTRIP_CASES + ('Hi.\r\nThis is a test.\r\nTesting.\r\n', '\nHi.\r\nThis is a test.\n\r\nTesting.\r\n\n')
for text in ROUNDTRIP_CASES:

    assert dedent(indent(text, ' \t  \t ')) == text
print("IndentTestCase::test_roundtrip_mixed: ok")
"###);
    assert_output(&out, r###"IndentTestCase::test_roundtrip_mixed: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/indent_test_case__test_roundtrip_spaces.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_indent_test_case__test_roundtrip_spaces() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "indent_test_case__test_roundtrip_spaces"
# subject = "cpython.test_textwrap.IndentTestCase.test_roundtrip_spaces"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::IndentTestCase::test_roundtrip_spaces
"""Auto-ported test: IndentTestCase::test_roundtrip_spaces (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


class BaseTestCase(unittest.TestCase):
    """Parent class with utility methods for textwrap tests."""

    def show(self, textin):
        if isinstance(textin, list):
            result = []
            for i in range(len(textin)):
                result.append('  %d: %r' % (i, textin[i]))
            result = '\n'.join(result) if result else '  no lines'
        elif isinstance(textin, str):
            result = '  %s\n' % repr(textin)
        return result

    def check(self, result, expect):
        self.assertEqual(result, expect, 'expected:\n%s\nbut got:\n%s' % (self.show(expect), self.show(result)))

    def check_wrap(self, text, width, expect, **kwargs):
        result = wrap(text, width, **kwargs)
        self.check(result, expect)

    def check_split(self, text, expect):
        result = self.wrapper._split(text)
        self.assertEqual(result, expect, '\nexpected %r\nbut got  %r' % (expect, result))


# --- test body ---
ROUNDTRIP_CASES = ('Hi.\nThis is a test.\nTesting.', 'Hi.\nThis is a test.\n\nTesting.', '\nHi.\nThis is a test.\nTesting.\n')
CASES = ROUNDTRIP_CASES + ('Hi.\r\nThis is a test.\r\nTesting.\r\n', '\nHi.\r\nThis is a test.\n\r\nTesting.\r\n\n')
for text in ROUNDTRIP_CASES:

    assert dedent(indent(text, '    ')) == text
print("IndentTestCase::test_roundtrip_spaces: ok")
"###);
    assert_output(&out, r###"IndentTestCase::test_roundtrip_spaces: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/indent_test_case__test_roundtrip_tabs.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_indent_test_case__test_roundtrip_tabs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "indent_test_case__test_roundtrip_tabs"
# subject = "cpython.test_textwrap.IndentTestCase.test_roundtrip_tabs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::IndentTestCase::test_roundtrip_tabs
"""Auto-ported test: IndentTestCase::test_roundtrip_tabs (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


class BaseTestCase(unittest.TestCase):
    """Parent class with utility methods for textwrap tests."""

    def show(self, textin):
        if isinstance(textin, list):
            result = []
            for i in range(len(textin)):
                result.append('  %d: %r' % (i, textin[i]))
            result = '\n'.join(result) if result else '  no lines'
        elif isinstance(textin, str):
            result = '  %s\n' % repr(textin)
        return result

    def check(self, result, expect):
        self.assertEqual(result, expect, 'expected:\n%s\nbut got:\n%s' % (self.show(expect), self.show(result)))

    def check_wrap(self, text, width, expect, **kwargs):
        result = wrap(text, width, **kwargs)
        self.check(result, expect)

    def check_split(self, text, expect):
        result = self.wrapper._split(text)
        self.assertEqual(result, expect, '\nexpected %r\nbut got  %r' % (expect, result))


# --- test body ---
ROUNDTRIP_CASES = ('Hi.\nThis is a test.\nTesting.', 'Hi.\nThis is a test.\n\nTesting.', '\nHi.\nThis is a test.\nTesting.\n')
CASES = ROUNDTRIP_CASES + ('Hi.\r\nThis is a test.\r\nTesting.\r\n', '\nHi.\r\nThis is a test.\n\r\nTesting.\r\n\n')
for text in ROUNDTRIP_CASES:

    assert dedent(indent(text, '\t\t')) == text
print("IndentTestCase::test_roundtrip_tabs: ok")
"###);
    assert_output(&out, r###"IndentTestCase::test_roundtrip_tabs: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/indent_test_cases__test_fill.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_indent_test_cases__test_fill() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "indent_test_cases__test_fill"
# subject = "cpython.test_textwrap.IndentTestCases.test_fill"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::IndentTestCases::test_fill
"""Auto-ported test: IndentTestCases::test_fill (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_text = 'This paragraph will be filled, first without any indentation,\nand then with some (including a hanging indent).'
expect = 'This paragraph will be filled, first\nwithout any indentation, and then with\nsome (including a hanging indent).'
result = fill(self_text, 40)
check(result, expect)
print("IndentTestCases::test_fill: ok")
"###);
    assert_output(&out, r###"IndentTestCases::test_fill: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/long_word_test_case__test_break_long.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_long_word_test_case__test_break_long() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "long_word_test_case__test_break_long"
# subject = "cpython.test_textwrap.LongWordTestCase.test_break_long"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::LongWordTestCase::test_break_long
"""Auto-ported test: LongWordTestCase::test_break_long (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper()
self_text = 'Did you say "supercalifragilisticexpialidocious?"\nHow *do* you spell that odd word, anyways?\n'
check_wrap(self_text, 30, ['Did you say "supercalifragilis', 'ticexpialidocious?" How *do*', 'you spell that odd word,', 'anyways?'])
check_wrap(self_text, 50, ['Did you say "supercalifragilisticexpialidocious?"', 'How *do* you spell that odd word, anyways?'])
check_wrap('-' * 10 + 'hello', 10, ['----------', '               h', '               e', '               l', '               l', '               o'], subsequent_indent=' ' * 15)
check_wrap(self_text, 12, ['Did you say ', '"supercalifr', 'agilisticexp', 'ialidocious?', '" How *do*', 'you spell', 'that odd', 'word,', 'anyways?'])
print("LongWordTestCase::test_break_long: ok")
"###);
    assert_output(&out, r###"LongWordTestCase::test_break_long: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/long_word_test_case__test_max_lines_long.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_long_word_test_case__test_max_lines_long() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "long_word_test_case__test_max_lines_long"
# subject = "cpython.test_textwrap.LongWordTestCase.test_max_lines_long"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::LongWordTestCase::test_max_lines_long
"""Auto-ported test: LongWordTestCase::test_max_lines_long (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper()
self_text = 'Did you say "supercalifragilisticexpialidocious?"\nHow *do* you spell that odd word, anyways?\n'
check_wrap(self_text, 12, ['Did you say ', '"supercalifr', 'agilisticexp', '[...]'], max_lines=4)
print("LongWordTestCase::test_max_lines_long: ok")
"###);
    assert_output(&out, r###"LongWordTestCase::test_max_lines_long: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/long_word_with_hyphens_test_case__test_break_long_words_not_on_hyphen.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_long_word_with_hyphens_test_case__test_break_long_words_not_on_hyphen() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "long_word_with_hyphens_test_case__test_break_long_words_not_on_hyphen"
# subject = "cpython.test_textwrap.LongWordWithHyphensTestCase.test_break_long_words_not_on_hyphen"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::LongWordWithHyphensTestCase::test_break_long_words_not_on_hyphen
"""Auto-ported test: LongWordWithHyphensTestCase::test_break_long_words_not_on_hyphen (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper()
self_text1 = 'We used enyzme 2-succinyl-6-hydroxy-2,4-cyclohexadiene-1-carboxylate synthase.\n'
self_text2 = '1234567890-1234567890--this_is_a_very_long_option_indeed-good-bye"\n'
expected = ['We used enyzme 2-succinyl-6-hydroxy-2,4-cyclohexad', 'iene-1-carboxylate synthase.']
check_wrap(self_text1, 50, expected, break_on_hyphens=False)
expected = ['We used', 'enyzme 2-s', 'uccinyl-6-', 'hydroxy-2,', '4-cyclohex', 'adiene-1-c', 'arboxylate', 'synthase.']
check_wrap(self_text1, 10, expected, break_on_hyphens=False)
expected = ['1234567890', '-123456789', '0--this_is', '_a_very_lo', 'ng_option_', 'indeed-', 'good-bye"']
check_wrap(self_text2, 10, expected)
print("LongWordWithHyphensTestCase::test_break_long_words_not_on_hyphen: ok")
"###);
    assert_output(&out, r###"LongWordWithHyphensTestCase::test_break_long_words_not_on_hyphen: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/long_word_with_hyphens_test_case__test_break_long_words_on_hyphen.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_long_word_with_hyphens_test_case__test_break_long_words_on_hyphen() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "long_word_with_hyphens_test_case__test_break_long_words_on_hyphen"
# subject = "cpython.test_textwrap.LongWordWithHyphensTestCase.test_break_long_words_on_hyphen"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::LongWordWithHyphensTestCase::test_break_long_words_on_hyphen
"""Auto-ported test: LongWordWithHyphensTestCase::test_break_long_words_on_hyphen (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper()
self_text1 = 'We used enyzme 2-succinyl-6-hydroxy-2,4-cyclohexadiene-1-carboxylate synthase.\n'
self_text2 = '1234567890-1234567890--this_is_a_very_long_option_indeed-good-bye"\n'
expected = ['We used enyzme 2-succinyl-6-hydroxy-2,4-', 'cyclohexadiene-1-carboxylate synthase.']
check_wrap(self_text1, 50, expected)
expected = ['We used', 'enyzme 2-', 'succinyl-', '6-hydroxy-', '2,4-', 'cyclohexad', 'iene-1-', 'carboxylat', 'e', 'synthase.']
check_wrap(self_text1, 10, expected)
expected = ['1234567890', '-123456789', '0--this_is', '_a_very_lo', 'ng_option_', 'indeed-', 'good-bye"']
check_wrap(self_text2, 10, expected)
print("LongWordWithHyphensTestCase::test_break_long_words_on_hyphen: ok")
"###);
    assert_output(&out, r###"LongWordWithHyphensTestCase::test_break_long_words_on_hyphen: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/long_word_with_hyphens_test_case__test_break_on_hyphen_but_not_long_words.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_long_word_with_hyphens_test_case__test_break_on_hyphen_but_not_long_words() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "long_word_with_hyphens_test_case__test_break_on_hyphen_but_not_long_words"
# subject = "cpython.test_textwrap.LongWordWithHyphensTestCase.test_break_on_hyphen_but_not_long_words"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::LongWordWithHyphensTestCase::test_break_on_hyphen_but_not_long_words
"""Auto-ported test: LongWordWithHyphensTestCase::test_break_on_hyphen_but_not_long_words (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper()
self_text1 = 'We used enyzme 2-succinyl-6-hydroxy-2,4-cyclohexadiene-1-carboxylate synthase.\n'
self_text2 = '1234567890-1234567890--this_is_a_very_long_option_indeed-good-bye"\n'
expected = ['We used enyzme', '2-succinyl-6-hydroxy-2,4-cyclohexadiene-1-carboxylate', 'synthase.']
check_wrap(self_text1, 50, expected, break_long_words=False)
expected = ['We used', 'enyzme', '2-succinyl-6-hydroxy-2,4-cyclohexadiene-1-carboxylate', 'synthase.']
check_wrap(self_text1, 10, expected, break_long_words=False)
expected = ['1234567890', '-123456789', '0--this_is', '_a_very_lo', 'ng_option_', 'indeed-', 'good-bye"']
check_wrap(self_text2, 10, expected)
print("LongWordWithHyphensTestCase::test_break_on_hyphen_but_not_long_words: ok")
"###);
    assert_output(&out, r###"LongWordWithHyphensTestCase::test_break_on_hyphen_but_not_long_words: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/long_word_with_hyphens_test_case__test_do_not_break_long_words_or_on_hyphens.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_long_word_with_hyphens_test_case__test_do_not_break_long_words_or_on_hyphens() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "long_word_with_hyphens_test_case__test_do_not_break_long_words_or_on_hyphens"
# subject = "cpython.test_textwrap.LongWordWithHyphensTestCase.test_do_not_break_long_words_or_on_hyphens"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::LongWordWithHyphensTestCase::test_do_not_break_long_words_or_on_hyphens
"""Auto-ported test: LongWordWithHyphensTestCase::test_do_not_break_long_words_or_on_hyphens (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper()
self_text1 = 'We used enyzme 2-succinyl-6-hydroxy-2,4-cyclohexadiene-1-carboxylate synthase.\n'
self_text2 = '1234567890-1234567890--this_is_a_very_long_option_indeed-good-bye"\n'
expected = ['We used enyzme', '2-succinyl-6-hydroxy-2,4-cyclohexadiene-1-carboxylate', 'synthase.']
check_wrap(self_text1, 50, expected, break_long_words=False, break_on_hyphens=False)
expected = ['We used', 'enyzme', '2-succinyl-6-hydroxy-2,4-cyclohexadiene-1-carboxylate', 'synthase.']
check_wrap(self_text1, 10, expected, break_long_words=False, break_on_hyphens=False)
expected = ['1234567890', '-123456789', '0--this_is', '_a_very_lo', 'ng_option_', 'indeed-', 'good-bye"']
check_wrap(self_text2, 10, expected)
print("LongWordWithHyphensTestCase::test_do_not_break_long_words_or_on_hyphens: ok")
"###);
    assert_output(&out, r###"LongWordWithHyphensTestCase::test_do_not_break_long_words_or_on_hyphens: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/max_lines_test_case__test_placeholder_backtrack.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_max_lines_test_case__test_placeholder_backtrack() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "max_lines_test_case__test_placeholder_backtrack"
# subject = "cpython.test_textwrap.MaxLinesTestCase.test_placeholder_backtrack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::MaxLinesTestCase::test_placeholder_backtrack
"""Auto-ported test: MaxLinesTestCase::test_placeholder_backtrack (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
text = "Hello there, how are you this fine day?  I'm glad to hear it!"

def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
text = 'Good grief Python features are advancing quickly!'
check_wrap(text, 12, ['Good grief', 'Python*****'], max_lines=3, placeholder='*****')
print("MaxLinesTestCase::test_placeholder_backtrack: ok")
"###);
    assert_output(&out, r###"MaxLinesTestCase::test_placeholder_backtrack: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/max_lines_test_case__test_simple.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_max_lines_test_case__test_simple() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "max_lines_test_case__test_simple"
# subject = "cpython.test_textwrap.MaxLinesTestCase.test_simple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::MaxLinesTestCase::test_simple
"""Auto-ported test: MaxLinesTestCase::test_simple (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
text = "Hello there, how are you this fine day?  I'm glad to hear it!"

def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
check_wrap(text, 12, ['Hello [...]'], max_lines=0)
check_wrap(text, 12, ['Hello [...]'], max_lines=1)
check_wrap(text, 12, ['Hello there,', 'how [...]'], max_lines=2)
check_wrap(text, 13, ['Hello there,', 'how are [...]'], max_lines=2)
check_wrap(text, 80, [text], max_lines=1)
check_wrap(text, 12, ['Hello there,', 'how are you', 'this fine', "day?  I'm", 'glad to hear', 'it!'], max_lines=6)
print("MaxLinesTestCase::test_simple: ok")
"###);
    assert_output(&out, r###"MaxLinesTestCase::test_simple: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/max_lines_test_case__test_spaces.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_max_lines_test_case__test_spaces() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "max_lines_test_case__test_spaces"
# subject = "cpython.test_textwrap.MaxLinesTestCase.test_spaces"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::MaxLinesTestCase::test_spaces
"""Auto-ported test: MaxLinesTestCase::test_spaces (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
text = "Hello there, how are you this fine day?  I'm glad to hear it!"

def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
check_wrap(text, 12, ['Hello there,', 'how are you', 'this fine', 'day? [...]'], max_lines=4)
check_wrap(text, 6, ['Hello', '[...]'], max_lines=2)
check_wrap(text + ' ' * 10, 12, ['Hello there,', 'how are you', 'this fine', "day?  I'm", 'glad to hear', 'it!'], max_lines=6)
print("MaxLinesTestCase::test_spaces: ok")
"###);
    assert_output(&out, r###"MaxLinesTestCase::test_spaces: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/shorten_collapses_whitespace_and_truncates.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_shorten_collapses_whitespace_and_truncates() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "shorten_collapses_whitespace_and_truncates"
# subject = "textwrap.shorten"
# kind = "semantic"
# xfail = "textwrap.shorten is a silent stub under mamba — no whitespace-collapse/truncate (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.shorten: shorten collapses runs of whitespace and truncates with the placeholder so the result fits within width"""
import textwrap

sh = textwrap.shorten("   hello   world   ", width=10, placeholder="...")
assert sh == "hello...", f"shorten = {sh!r}"
assert len(sh) <= 10, f"shorten len = {len(sh)!r}"
# A longer sentence truncates with the placeholder and stays within width.
sh2 = textwrap.shorten("hello world this is long", width=15, placeholder="...")
assert len(sh2) <= 15, f"shorten len = {len(sh2)!r}"
assert sh2.endswith("..."), f"shorten ends with ... = {sh2!r}"
print("shorten_collapses_whitespace_and_truncates OK")
"###);
    assert_output(&out, r###"shorten_collapses_whitespace_and_truncates OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/shorten_placeholder_exact_width_allowed.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_shorten_placeholder_exact_width_allowed() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "shorten_placeholder_exact_width_allowed"
# subject = "textwrap.shorten"
# kind = "semantic"
# xfail = "textwrap.shorten is a silent stub under mamba — does not return the placeholder (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.shorten: a placeholder exactly as wide as width is accepted and returned when the whole text is collapsed away"""
import textwrap

# An 8-char placeholder at width=8 is the boundary case: accepted and returned.
result = textwrap.shorten("x" * 20, width=8, placeholder="(......)")
assert result == "(......)", f"placeholder of exactly width chars is allowed: {result!r}"
print("shorten_placeholder_exact_width_allowed OK")
"###);
    assert_output(&out, r###"shorten_placeholder_exact_width_allowed OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/shorten_test_case__test_empty_string.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_shorten_test_case__test_empty_string() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "shorten_test_case__test_empty_string"
# subject = "cpython.test_textwrap.ShortenTestCase.test_empty_string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::ShortenTestCase::test_empty_string
"""Auto-ported test: ShortenTestCase::test_empty_string (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_shorten(text, width, expect, **kwargs):
    result = shorten(text, width, **kwargs)
    check(result, expect)

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
check_shorten('', 6, '')
print("ShortenTestCase::test_empty_string: ok")
"###);
    assert_output(&out, r###"ShortenTestCase::test_empty_string: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/shorten_test_case__test_first_word_too_long_but_placeholder_fits.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_shorten_test_case__test_first_word_too_long_but_placeholder_fits() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "shorten_test_case__test_first_word_too_long_but_placeholder_fits"
# subject = "cpython.test_textwrap.ShortenTestCase.test_first_word_too_long_but_placeholder_fits"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::ShortenTestCase::test_first_word_too_long_but_placeholder_fits
"""Auto-ported test: ShortenTestCase::test_first_word_too_long_but_placeholder_fits (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_shorten(text, width, expect, **kwargs):
    result = shorten(text, width, **kwargs)
    check(result, expect)

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
check_shorten('Helloo', 5, '[...]')
print("ShortenTestCase::test_first_word_too_long_but_placeholder_fits: ok")
"###);
    assert_output(&out, r###"ShortenTestCase::test_first_word_too_long_but_placeholder_fits: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/shorten_test_case__test_placeholder.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_shorten_test_case__test_placeholder() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "shorten_test_case__test_placeholder"
# subject = "cpython.test_textwrap.ShortenTestCase.test_placeholder"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::ShortenTestCase::test_placeholder
"""Auto-ported test: ShortenTestCase::test_placeholder (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_shorten(text, width, expect, **kwargs):
    result = shorten(text, width, **kwargs)
    check(result, expect)

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
text = "Hello there, how are you this fine day? I'm glad to hear it!"
check_shorten(text, 17, 'Hello there,$$', placeholder='$$')
check_shorten(text, 18, 'Hello there, how$$', placeholder='$$')
check_shorten(text, 18, 'Hello there, $$', placeholder=' $$')
check_shorten(text, len(text), text, placeholder='$$')
check_shorten(text, len(text) - 1, "Hello there, how are you this fine day? I'm glad to hear$$", placeholder='$$')
print("ShortenTestCase::test_placeholder: ok")
"###);
    assert_output(&out, r###"ShortenTestCase::test_placeholder: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/shorten_test_case__test_simple.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_shorten_test_case__test_simple() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "shorten_test_case__test_simple"
# subject = "cpython.test_textwrap.ShortenTestCase.test_simple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::ShortenTestCase::test_simple
"""Auto-ported test: ShortenTestCase::test_simple (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_shorten(text, width, expect, **kwargs):
    result = shorten(text, width, **kwargs)
    check(result, expect)

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
text = "Hello there, how are you this fine day? I'm glad to hear it!"
check_shorten(text, 18, 'Hello there, [...]')
check_shorten(text, len(text), text)
check_shorten(text, len(text) - 1, "Hello there, how are you this fine day? I'm glad to [...]")
print("ShortenTestCase::test_simple: ok")
"###);
    assert_output(&out, r###"ShortenTestCase::test_simple: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/shorten_test_case__test_whitespace.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_shorten_test_case__test_whitespace() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "shorten_test_case__test_whitespace"
# subject = "cpython.test_textwrap.ShortenTestCase.test_whitespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::ShortenTestCase::test_whitespace
"""Auto-ported test: ShortenTestCase::test_whitespace (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_shorten(text, width, expect, **kwargs):
    result = shorten(text, width, **kwargs)
    check(result, expect)

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
text = '\n            This is a  paragraph that  already has\n            line breaks and \t tabs too.'
check_shorten(text, 62, 'This is a paragraph that already has line breaks and tabs too.')
check_shorten(text, 61, 'This is a paragraph that already has line breaks and [...]')
check_shorten('hello      world!  ', 12, 'hello world!')
check_shorten('hello      world!  ', 11, 'hello [...]')
check_shorten('hello      world!  ', 10, '[...]')
print("ShortenTestCase::test_whitespace: ok")
"###);
    assert_output(&out, r###"ShortenTestCase::test_whitespace: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/textwrapper_initial_and_subsequent_indent.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_textwrapper_initial_and_subsequent_indent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "textwrapper_initial_and_subsequent_indent"
# subject = "textwrap.TextWrapper"
# kind = "semantic"
# xfail = "textwrap.TextWrapper.fill is a silent stub under mamba — no wrap/indent applied (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.TextWrapper: TextWrapper applies initial_indent to the first wrapped line and subsequent_indent to the rest"""
import textwrap

wrapper = textwrap.TextWrapper(width=20, initial_indent=">> ", subsequent_indent="   ")
result = wrapper.fill("hello world this is a long sentence")
lines = result.split("\n")
assert len(lines) > 1, f"expected multiple wrapped lines, got {lines!r}"
assert lines[0].startswith(">> "), f"initial_indent = {lines[0]!r}"
assert lines[1].startswith("   "), f"subsequent_indent = {lines[1]!r}"
print("textwrapper_initial_and_subsequent_indent OK")
"###);
    assert_output(&out, r###"textwrapper_initial_and_subsequent_indent OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/textwrapper_width_bounds_lines.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_textwrapper_width_bounds_lines() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "textwrapper_width_bounds_lines"
# subject = "textwrap.TextWrapper"
# kind = "semantic"
# xfail = "textwrap.TextWrapper.wrap is a silent stub under mamba — does not split to width (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.TextWrapper: a TextWrapper(width=20).wrap returns a list of strings each within the configured width"""
import textwrap

wrapper = textwrap.TextWrapper(width=20)
assert wrapper.width == 20, f"TextWrapper.width = {wrapper.width!r}"
wrapped = wrapper.wrap("the quick brown fox jumps over the lazy dog")
assert isinstance(wrapped, list), f"TextWrapper.wrap type = {type(wrapped)!r}"
assert len(wrapped) > 1, f"expected multiple lines, got {wrapped!r}"
assert all(len(s) <= 20 for s in wrapped), f"all within width = {wrapped!r}"
print("textwrapper_width_bounds_lines OK")
"###);
    assert_output(&out, r###"textwrapper_width_bounds_lines OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_break_long_words_false_keeps_word_intact.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_break_long_words_false_keeps_word_intact() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_break_long_words_false_keeps_word_intact"
# subject = "textwrap.wrap"
# kind = "semantic"
# xfail = "textwrap.wrap is a silent stub under mamba — does not split to width (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.wrap: with break_long_words=False, a word longer than width stays intact on its own line"""
import textwrap

nobreak = textwrap.wrap(
    "short superlongwordthatexceedswidth end", width=10, break_long_words=False
)
assert any("superlongwordthatexceedswidth" in line for line in nobreak), (
    f"long word preserved = {nobreak!r}"
)
# Also with the simpler input from the legacy monolith.
nobreak2 = textwrap.wrap("ab longer_word cd", width=10, break_long_words=False)
assert any("longer_word" in line for line in nobreak2), f"long word intact = {nobreak2!r}"
print("wrap_break_long_words_false_keeps_word_intact OK")
"###);
    assert_output(&out, r###"wrap_break_long_words_false_keeps_word_intact OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_breaks_lines_within_width.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_breaks_lines_within_width() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_breaks_lines_within_width"
# subject = "textwrap.wrap"
# kind = "semantic"
# xfail = "textwrap.wrap is a silent stub under mamba — does not split to width (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.wrap: wrap returns a list of strings each within width and splits a long word when break_long_words=True (default)"""
import textwrap

words = "ab longer_word cd"
wrapped = textwrap.wrap(words, width=10)
assert isinstance(wrapped, list), f"wrap type = {type(wrapped)!r}"
assert all(isinstance(s, str) for s in wrapped), "all strings"
# "longer_word" (11 chars) exceeds width and is split by default.
assert all(len(line) <= 10 for line in wrapped), f"lines within width = {wrapped!r}"
print("wrap_breaks_lines_within_width OK")
"###);
    assert_output(&out, r###"wrap_breaks_lines_within_width OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_empty_string_returns_empty_list.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_empty_string_returns_empty_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_empty_string_returns_empty_list"
# subject = "textwrap.wrap"
# kind = "semantic"
# xfail = "textwrap.wrap is a silent stub under mamba — returns input rather than [] (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.wrap: wrap on the empty string returns an empty list"""
import textwrap

assert textwrap.wrap("") == [], f"wrap('') = {textwrap.wrap('')!r}"
print("wrap_empty_string_returns_empty_list OK")
"###);
    assert_output(&out, r###"wrap_empty_string_returns_empty_list OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_bad_width.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_bad_width() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_bad_width"
# subject = "cpython.test_textwrap.WrapTestCase.test_bad_width"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_bad_width
"""Auto-ported test: WrapTestCase::test_bad_width (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


class BaseTestCase(unittest.TestCase):
    """Parent class with utility methods for textwrap tests."""

    def show(self, textin):
        if isinstance(textin, list):
            result = []
            for i in range(len(textin)):
                result.append('  %d: %r' % (i, textin[i]))
            result = '\n'.join(result) if result else '  no lines'
        elif isinstance(textin, str):
            result = '  %s\n' % repr(textin)
        return result

    def check(self, result, expect):
        self.assertEqual(result, expect, 'expected:\n%s\nbut got:\n%s' % (self.show(expect), self.show(result)))

    def check_wrap(self, text, width, expect, **kwargs):
        result = wrap(text, width, **kwargs)
        self.check(result, expect)

    def check_split(self, text, expect):
        result = self.wrapper._split(text)
        self.assertEqual(result, expect, '\nexpected %r\nbut got  %r' % (expect, result))


# --- test body ---
self_wrapper = TextWrapper(width=45)
text = "Whatever, it doesn't matter."

try:
    wrap(text, 0)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    wrap(text, -1)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("WrapTestCase::test_bad_width: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_bad_width: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_break_on_hyphens.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_break_on_hyphens() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_break_on_hyphens"
# subject = "cpython.test_textwrap.WrapTestCase.test_break_on_hyphens"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_break_on_hyphens
"""Auto-ported test: WrapTestCase::test_break_on_hyphens (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper(width=45)
text = 'yaba daba-doo'
check_wrap(text, 10, ['yaba daba-', 'doo'], break_on_hyphens=True)
check_wrap(text, 10, ['yaba', 'daba-doo'], break_on_hyphens=False)
print("WrapTestCase::test_break_on_hyphens: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_break_on_hyphens: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_drop_whitespace_false.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_drop_whitespace_false() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_drop_whitespace_false"
# subject = "cpython.test_textwrap.WrapTestCase.test_drop_whitespace_false"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_drop_whitespace_false
"""Auto-ported test: WrapTestCase::test_drop_whitespace_false (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper(width=45)
text = ' This is a    sentence with     much whitespace.'
check_wrap(text, 10, [' This is a', '    ', 'sentence ', 'with     ', 'much white', 'space.'], drop_whitespace=False)
print("WrapTestCase::test_drop_whitespace_false: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_drop_whitespace_false: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_drop_whitespace_false_whitespace_only.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_drop_whitespace_false_whitespace_only() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_drop_whitespace_false_whitespace_only"
# subject = "cpython.test_textwrap.WrapTestCase.test_drop_whitespace_false_whitespace_only"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_drop_whitespace_false_whitespace_only
"""Auto-ported test: WrapTestCase::test_drop_whitespace_false_whitespace_only (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper(width=45)
check_wrap('   ', 6, ['   '], drop_whitespace=False)
print("WrapTestCase::test_drop_whitespace_false_whitespace_only: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_drop_whitespace_false_whitespace_only: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_drop_whitespace_false_whitespace_only_with_indent.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_drop_whitespace_false_whitespace_only_with_indent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_drop_whitespace_false_whitespace_only_with_indent"
# subject = "cpython.test_textwrap.WrapTestCase.test_drop_whitespace_false_whitespace_only_with_indent"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_drop_whitespace_false_whitespace_only_with_indent
"""Auto-ported test: WrapTestCase::test_drop_whitespace_false_whitespace_only_with_indent (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper(width=45)
check_wrap('   ', 6, ['     '], drop_whitespace=False, initial_indent='  ')
print("WrapTestCase::test_drop_whitespace_false_whitespace_only_with_indent: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_drop_whitespace_false_whitespace_only_with_indent: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_drop_whitespace_leading_whitespace.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_drop_whitespace_leading_whitespace() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_drop_whitespace_leading_whitespace"
# subject = "cpython.test_textwrap.WrapTestCase.test_drop_whitespace_leading_whitespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_drop_whitespace_leading_whitespace
"""Auto-ported test: WrapTestCase::test_drop_whitespace_leading_whitespace (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper(width=45)
text = ' This is a sentence with leading whitespace.'
check_wrap(text, 50, [' This is a sentence with leading whitespace.'])
check_wrap(text, 30, [' This is a sentence with', 'leading whitespace.'])
print("WrapTestCase::test_drop_whitespace_leading_whitespace: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_drop_whitespace_leading_whitespace: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_drop_whitespace_whitespace_indent.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_drop_whitespace_whitespace_indent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_drop_whitespace_whitespace_indent"
# subject = "cpython.test_textwrap.WrapTestCase.test_drop_whitespace_whitespace_indent"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_drop_whitespace_whitespace_indent
"""Auto-ported test: WrapTestCase::test_drop_whitespace_whitespace_indent (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper(width=45)
check_wrap('abcd efgh', 6, ['  abcd', '  efgh'], initial_indent='  ', subsequent_indent='  ')
print("WrapTestCase::test_drop_whitespace_whitespace_indent: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_drop_whitespace_whitespace_indent: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_drop_whitespace_whitespace_line.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_drop_whitespace_whitespace_line() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_drop_whitespace_whitespace_line"
# subject = "cpython.test_textwrap.WrapTestCase.test_drop_whitespace_whitespace_line"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_drop_whitespace_whitespace_line
"""Auto-ported test: WrapTestCase::test_drop_whitespace_whitespace_line (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper(width=45)
text = 'abcd    efgh'
check_wrap(text, 6, ['abcd', '    ', 'efgh'], drop_whitespace=False)
check_wrap(text, 6, ['abcd', 'efgh'])
print("WrapTestCase::test_drop_whitespace_whitespace_line: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_drop_whitespace_whitespace_line: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_drop_whitespace_whitespace_only.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_drop_whitespace_whitespace_only() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_drop_whitespace_whitespace_only"
# subject = "cpython.test_textwrap.WrapTestCase.test_drop_whitespace_whitespace_only"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_drop_whitespace_whitespace_only
"""Auto-ported test: WrapTestCase::test_drop_whitespace_whitespace_only (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper(width=45)
check_wrap('  ', 6, [])
print("WrapTestCase::test_drop_whitespace_whitespace_only: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_drop_whitespace_whitespace_only: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_drop_whitespace_whitespace_only_with_indent.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_drop_whitespace_whitespace_only_with_indent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_drop_whitespace_whitespace_only_with_indent"
# subject = "cpython.test_textwrap.WrapTestCase.test_drop_whitespace_whitespace_only_with_indent"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_drop_whitespace_whitespace_only_with_indent
"""Auto-ported test: WrapTestCase::test_drop_whitespace_whitespace_only_with_indent (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper(width=45)
check_wrap('  ', 6, [], initial_indent='++')
print("WrapTestCase::test_drop_whitespace_whitespace_only_with_indent: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_drop_whitespace_whitespace_only_with_indent: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_em_dash.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_em_dash() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_em_dash"
# subject = "cpython.test_textwrap.WrapTestCase.test_em_dash"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_em_dash
"""Auto-ported test: WrapTestCase::test_em_dash (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper(width=45)
text = 'Em-dashes should be written -- thus.'
check_wrap(text, 25, ['Em-dashes should be', 'written -- thus.'])
check_wrap(text, 29, ['Em-dashes should be written', '-- thus.'])
expect = ['Em-dashes should be written --', 'thus.']
check_wrap(text, 30, expect)
check_wrap(text, 35, expect)
check_wrap(text, 36, ['Em-dashes should be written -- thus.'])
text = 'You can also do--this or even---this.'
expect = ['You can also do', '--this or even', '---this.']
check_wrap(text, 15, expect)
check_wrap(text, 16, expect)
expect = ['You can also do--', 'this or even---', 'this.']
check_wrap(text, 17, expect)
check_wrap(text, 19, expect)
expect = ['You can also do--this or even', '---this.']
check_wrap(text, 29, expect)
check_wrap(text, 31, expect)
expect = ['You can also do--this or even---', 'this.']
check_wrap(text, 32, expect)
check_wrap(text, 35, expect)
text = "Here's an -- em-dash and--here's another---and another!"
expect = ["Here's", ' ', 'an', ' ', '--', ' ', 'em-', 'dash', ' ', 'and', '--', "here's", ' ', 'another', '---', 'and', ' ', 'another!']
check_split(text, expect)
text = 'and then--bam!--he was gone'
expect = ['and', ' ', 'then', '--', 'bam!', '--', 'he', ' ', 'was', ' ', 'gone']
check_split(text, expect)
print("WrapTestCase::test_em_dash: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_em_dash: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_empty_string.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_empty_string() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_empty_string"
# subject = "cpython.test_textwrap.WrapTestCase.test_empty_string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_empty_string
"""Auto-ported test: WrapTestCase::test_empty_string (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper(width=45)
check_wrap('', 6, [])
check_wrap('', 6, [], drop_whitespace=False)
print("WrapTestCase::test_empty_string: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_empty_string: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_empty_string_with_initial_indent.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_empty_string_with_initial_indent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_empty_string_with_initial_indent"
# subject = "cpython.test_textwrap.WrapTestCase.test_empty_string_with_initial_indent"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_empty_string_with_initial_indent
"""Auto-ported test: WrapTestCase::test_empty_string_with_initial_indent (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper(width=45)
check_wrap('', 6, [], initial_indent='++')
check_wrap('', 6, [], initial_indent='++', drop_whitespace=False)
print("WrapTestCase::test_empty_string_with_initial_indent: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_empty_string_with_initial_indent: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_funky_hyphens.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_funky_hyphens() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_funky_hyphens"
# subject = "cpython.test_textwrap.WrapTestCase.test_funky_hyphens"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_funky_hyphens
"""Auto-ported test: WrapTestCase::test_funky_hyphens (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper(width=45)
check_split('what the--hey!', ['what', ' ', 'the', '--', 'hey!'])
check_split('what the--', ['what', ' ', 'the--'])
check_split('what the--.', ['what', ' ', 'the--.'])
check_split('--text--.', ['--text--.'])
check_split('--option', ['--option'])
check_split('--option-opt', ['--option-', 'opt'])
check_split('foo --option-opt bar', ['foo', ' ', '--option-', 'opt', ' ', 'bar'])
print("WrapTestCase::test_funky_hyphens: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_funky_hyphens: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_funky_parens.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_funky_parens() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_funky_parens"
# subject = "cpython.test_textwrap.WrapTestCase.test_funky_parens"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_funky_parens
"""Auto-ported test: WrapTestCase::test_funky_parens (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper(width=45)
check_split('foo (--option) bar', ['foo', ' ', '(--option)', ' ', 'bar'])
check_split('foo (bar) baz', ['foo', ' ', '(bar)', ' ', 'baz'])
check_split('blah (ding dong), wubba', ['blah', ' ', '(ding', ' ', 'dong),', ' ', 'wubba'])
print("WrapTestCase::test_funky_parens: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_funky_parens: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_hyphenated_numbers.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_hyphenated_numbers() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_hyphenated_numbers"
# subject = "cpython.test_textwrap.WrapTestCase.test_hyphenated_numbers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_hyphenated_numbers
"""Auto-ported test: WrapTestCase::test_hyphenated_numbers (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper(width=45)
text = 'Python 1.0.0 was released on 1994-01-26.  Python 1.0.1 was\nreleased on 1994-02-15.'
check_wrap(text, 30, ['Python 1.0.0 was released on', '1994-01-26.  Python 1.0.1 was', 'released on 1994-02-15.'])
check_wrap(text, 40, ['Python 1.0.0 was released on 1994-01-26.', 'Python 1.0.1 was released on 1994-02-15.'])
check_wrap(text, 1, text.split(), break_long_words=False)
text = 'I do all my shopping at 7-11.'
check_wrap(text, 25, ['I do all my shopping at', '7-11.'])
check_wrap(text, 27, ['I do all my shopping at', '7-11.'])
check_wrap(text, 29, ['I do all my shopping at 7-11.'])
check_wrap(text, 1, text.split(), break_long_words=False)
print("WrapTestCase::test_hyphenated_numbers: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_hyphenated_numbers: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_narrow_non_breaking_space.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_narrow_non_breaking_space() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_narrow_non_breaking_space"
# subject = "cpython.test_textwrap.WrapTestCase.test_narrow_non_breaking_space"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_narrow_non_breaking_space
"""Auto-ported test: WrapTestCase::test_narrow_non_breaking_space (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper(width=45)
text = 'This is a sentence with non-breaking\u202fspace.'
check_wrap(text, 20, ['This is a sentence', 'with non-', 'breaking\u202fspace.'], break_on_hyphens=True)
check_wrap(text, 20, ['This is a sentence', 'with', 'non-breaking\u202fspace.'], break_on_hyphens=False)
print("WrapTestCase::test_narrow_non_breaking_space: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_narrow_non_breaking_space: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_no_split_at_umlaut.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_no_split_at_umlaut() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_no_split_at_umlaut"
# subject = "cpython.test_textwrap.WrapTestCase.test_no_split_at_umlaut"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_no_split_at_umlaut
"""Auto-ported test: WrapTestCase::test_no_split_at_umlaut (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper(width=45)
text = 'Die Empfänger-Auswahl'
check_wrap(text, 13, ['Die', 'Empfänger-', 'Auswahl'])
print("WrapTestCase::test_no_split_at_umlaut: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_no_split_at_umlaut: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_non_breaking_space.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_non_breaking_space() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_non_breaking_space"
# subject = "cpython.test_textwrap.WrapTestCase.test_non_breaking_space"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_non_breaking_space
"""Auto-ported test: WrapTestCase::test_non_breaking_space (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper(width=45)
text = 'This is a sentence with non-breaking\xa0space.'
check_wrap(text, 20, ['This is a sentence', 'with non-', 'breaking\xa0space.'], break_on_hyphens=True)
check_wrap(text, 20, ['This is a sentence', 'with', 'non-breaking\xa0space.'], break_on_hyphens=False)
print("WrapTestCase::test_non_breaking_space: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_non_breaking_space: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_simple.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_simple() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_simple"
# subject = "cpython.test_textwrap.WrapTestCase.test_simple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_simple
"""Auto-ported test: WrapTestCase::test_simple (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper(width=45)
text = "Hello there, how are you this fine day?  I'm glad to hear it!"
check_wrap(text, 12, ['Hello there,', 'how are you', 'this fine', "day?  I'm", 'glad to hear', 'it!'])
check_wrap(text, 42, ['Hello there, how are you this fine day?', "I'm glad to hear it!"])
check_wrap(text, 80, [text])
print("WrapTestCase::test_simple: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_simple: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_split.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_split() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_split"
# subject = "cpython.test_textwrap.WrapTestCase.test_split"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_split
"""Auto-ported test: WrapTestCase::test_split (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper(width=45)
text = 'Hello there -- you goof-ball, use the -b option!'
result = self_wrapper._split(text)
check(result, ['Hello', ' ', 'there', ' ', '--', ' ', 'you', ' ', 'goof-', 'ball,', ' ', 'use', ' ', 'the', ' ', '-b', ' ', 'option!'])
print("WrapTestCase::test_split: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_split: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_umlaut_followed_by_dash.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_umlaut_followed_by_dash() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_umlaut_followed_by_dash"
# subject = "cpython.test_textwrap.WrapTestCase.test_umlaut_followed_by_dash"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_umlaut_followed_by_dash
"""Auto-ported test: WrapTestCase::test_umlaut_followed_by_dash (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper(width=45)
text = 'aa ää-ää'
check_wrap(text, 7, ['aa ää-', 'ää'])
print("WrapTestCase::test_umlaut_followed_by_dash: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_umlaut_followed_by_dash: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_unix_options.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_unix_options() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_unix_options"
# subject = "cpython.test_textwrap.WrapTestCase.test_unix_options"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_unix_options
"""Auto-ported test: WrapTestCase::test_unix_options (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper(width=45)
text = 'You should use the -n option, or --dry-run in its long form.'
check_wrap(text, 20, ['You should use the', '-n option, or --dry-', 'run in its long', 'form.'])
check_wrap(text, 21, ['You should use the -n', 'option, or --dry-run', 'in its long form.'])
expect = ['You should use the -n option, or', '--dry-run in its long form.']
check_wrap(text, 32, expect)
check_wrap(text, 34, expect)
check_wrap(text, 35, expect)
check_wrap(text, 38, expect)
expect = ['You should use the -n option, or --dry-', 'run in its long form.']
check_wrap(text, 39, expect)
check_wrap(text, 41, expect)
expect = ['You should use the -n option, or --dry-run', 'in its long form.']
check_wrap(text, 42, expect)
text = 'the -n option, or --dry-run or --dryrun'
expect = ['the', ' ', '-n', ' ', 'option,', ' ', 'or', ' ', '--dry-', 'run', ' ', 'or', ' ', '--dryrun']
check_split(text, expect)
print("WrapTestCase::test_unix_options: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_unix_options: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_wrap_short.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_wrap_short() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_wrap_short"
# subject = "cpython.test_textwrap.WrapTestCase.test_wrap_short"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_wrap_short
"""Auto-ported test: WrapTestCase::test_wrap_short (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper(width=45)
text = 'This is a\nshort paragraph.'
check_wrap(text, 20, ['This is a short', 'paragraph.'])
check_wrap(text, 40, ['This is a short paragraph.'])
print("WrapTestCase::test_wrap_short: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_wrap_short: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/textwrap/wrap_test_case__test_wrap_short_1line.py`.
#[test]
fn test_gen_behavior_std_libs_textwrap_wrap_test_case__test_wrap_short_1line() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_wrap_short_1line"
# subject = "cpython.test_textwrap.WrapTestCase.test_wrap_short_1line"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_wrap_short_1line
"""Auto-ported test: WrapTestCase::test_wrap_short_1line (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_wrapper = TextWrapper(width=45)
text = 'This is a short line.'
check_wrap(text, 30, ['This is a short line.'])
check_wrap(text, 30, ['(1) This is a short line.'], initial_indent='(1) ')
print("WrapTestCase::test_wrap_short_1line: ok")
"###);
    assert_output(&out, r###"WrapTestCase::test_wrap_short_1line: ok
"###);
}
