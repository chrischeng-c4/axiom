use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/re/bad_backref_raises.py`.
#[test]
fn test_gen_errors_std_libs_re_bad_backref_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "errors"
# case = "bad_backref_raises"
# subject = "re.compile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.compile: bad_backref_raises (errors)."""
import re

_raised = False
try:
    re.compile(r'(\1)')
except re.error:
    _raised = True
assert _raised, "bad_backref_raises: expected re.error"
print("bad_backref_raises OK")
"###);
    assert_output(&out, r###"bad_backref_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/re/bad_escape_raises.py`.
#[test]
fn test_gen_errors_std_libs_re_bad_escape_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "errors"
# case = "bad_escape_raises"
# subject = "re.compile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.compile: bad_escape_raises (errors)."""
import re

_raised = False
try:
    re.compile(r'\q')
except re.error:
    _raised = True
assert _raised, "bad_escape_raises: expected re.error"
print("bad_escape_raises OK")
"###);
    assert_output(&out, r###"bad_escape_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/re/bad_inline_flag_raises.py`.
#[test]
fn test_gen_errors_std_libs_re_bad_inline_flag_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "errors"
# case = "bad_inline_flag_raises"
# subject = "re.compile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.compile: bad_inline_flag_raises (errors)."""
import re

_raised = False
try:
    re.compile(r'(?z)')
except re.error:
    _raised = True
assert _raised, "bad_inline_flag_raises: expected re.error"
print("bad_inline_flag_raises OK")
"###);
    assert_output(&out, r###"bad_inline_flag_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/re/int_pattern_raises.py`.
#[test]
fn test_gen_errors_std_libs_re_int_pattern_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "errors"
# case = "int_pattern_raises"
# subject = "re.match"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.match: int_pattern_raises (errors)."""
import re

_raised = False
try:
    re.match(123, 'abc')
except TypeError:
    _raised = True
assert _raised, "int_pattern_raises: expected TypeError"
print("int_pattern_raises OK")
"###);
    assert_output(&out, r###"int_pattern_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/re/missing_group_index_raises.py`.
#[test]
fn test_gen_errors_std_libs_re_missing_group_index_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "errors"
# case = "missing_group_index_raises"
# subject = "re.Match.group"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.Match.group: missing_group_index_raises (errors)."""
import re

_raised = False
try:
    re.match(r'(a)(b)', 'ab').group(5)
except IndexError:
    _raised = True
assert _raised, "missing_group_index_raises: expected IndexError"
print("missing_group_index_raises OK")
"###);
    assert_output(&out, r###"missing_group_index_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/re/missing_group_name_raises.py`.
#[test]
fn test_gen_errors_std_libs_re_missing_group_name_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "errors"
# case = "missing_group_name_raises"
# subject = "re.Match.group"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.Match.group: missing_group_name_raises (errors)."""
import re

_raised = False
try:
    re.match(r'(a)(b)', 'ab').group('nope')
except IndexError:
    _raised = True
assert _raised, "missing_group_name_raises: expected IndexError"
print("missing_group_name_raises OK")
"###);
    assert_output(&out, r###"missing_group_name_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/re/multiple_repeat_raises.py`.
#[test]
fn test_gen_errors_std_libs_re_multiple_repeat_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "errors"
# case = "multiple_repeat_raises"
# subject = "re.compile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.compile: multiple_repeat_raises (errors)."""
import re

_raised = False
try:
    re.compile(r'a**')
except re.error:
    _raised = True
assert _raised, "multiple_repeat_raises: expected re.error"
print("multiple_repeat_raises OK")
"###);
    assert_output(&out, r###"multiple_repeat_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/re/nothing_to_repeat_raises.py`.
#[test]
fn test_gen_errors_std_libs_re_nothing_to_repeat_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "errors"
# case = "nothing_to_repeat_raises"
# subject = "re.compile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.compile: nothing_to_repeat_raises (errors)."""
import re

_raised = False
try:
    re.compile(r'*')
except re.error:
    _raised = True
assert _raised, "nothing_to_repeat_raises: expected re.error"
print("nothing_to_repeat_raises OK")
"###);
    assert_output(&out, r###"nothing_to_repeat_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/re/repeat_count_overflow_raises.py`.
#[test]
fn test_gen_errors_std_libs_re_repeat_count_overflow_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "errors"
# case = "repeat_count_overflow_raises"
# subject = "re.compile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.compile: repeat_count_overflow_raises (errors)."""
import re

_raised = False
try:
    re.compile('.{%d}' % (2 ** 100))
except OverflowError:
    _raised = True
assert _raised, "repeat_count_overflow_raises: expected OverflowError"
print("repeat_count_overflow_raises OK")
"###);
    assert_output(&out, r###"repeat_count_overflow_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/re/str_pattern_bytes_subject_raises.py`.
#[test]
fn test_gen_errors_std_libs_re_str_pattern_bytes_subject_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "errors"
# case = "str_pattern_bytes_subject_raises"
# subject = "re.Pattern.match"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.Pattern.match: a str-compiled pattern rejects a bytes subject with TypeError, and a bytes-compiled pattern rejects a str subject with TypeError"""
import re

str_pat = re.compile(".")
bytes_pat = re.compile(b".")

# A str pattern cannot scan a bytes subject.
try:
    str_pat.match(b"b")
    raise AssertionError("str pattern on bytes subject should raise TypeError")
except TypeError:
    pass

# A bytes pattern cannot scan a str subject.
try:
    bytes_pat.match("b")
    raise AssertionError("bytes pattern on str subject should raise TypeError")
except TypeError:
    pass

print("str_pattern_bytes_subject_raises OK")
"###);
    assert_output(&out, r###"str_pattern_bytes_subject_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/re/sub_replacement_type_mismatch_raises.py`.
#[test]
fn test_gen_errors_std_libs_re_sub_replacement_type_mismatch_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "errors"
# case = "sub_replacement_type_mismatch_raises"
# subject = "re.Pattern.sub"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.Pattern.sub: a str-compiled pattern's sub() rejects a bytes replacement against a str subject with TypeError"""
import re

str_pat = re.compile(".")

# A bytes replacement against a str subject is a type mismatch.
try:
    str_pat.sub(b"b", "c")
    raise AssertionError("bytes replacement on str subject should raise TypeError")
except TypeError:
    pass

print("sub_replacement_type_mismatch_raises OK")
"###);
    assert_output(&out, r###"sub_replacement_type_mismatch_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/re/unclosed_paren_raises.py`.
#[test]
fn test_gen_errors_std_libs_re_unclosed_paren_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "errors"
# case = "unclosed_paren_raises"
# subject = "re.compile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.compile: unclosed_paren_raises (errors)."""
import re

_raised = False
try:
    re.compile(r'(unclosed')
except re.error:
    _raised = True
assert _raised, "unclosed_paren_raises: expected re.error"
print("unclosed_paren_raises OK")
"###);
    assert_output(&out, r###"unclosed_paren_raises OK
"###);
}
