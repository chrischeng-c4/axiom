use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/pep/634/dup_mapping_literal_key_syntaxerror.py`.
#[test]
fn test_gen_errors_pep_634_dup_mapping_literal_key_syntaxerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "errors"
# case = "dup_mapping_literal_key_syntaxerror"
# subject = "match.mapping_pattern"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.mapping_pattern: dup_mapping_literal_key_syntaxerror (errors)."""
pass

_raised = False
try:
    compile("match {}:\n case {'a': 1, 'a': 2}: pass", '<dup>', 'exec')
except SyntaxError:
    _raised = True
assert _raised, "dup_mapping_literal_key_syntaxerror: expected SyntaxError"
print("dup_mapping_literal_key_syntaxerror OK")
"###);
    assert_output(&out, r###"dup_mapping_literal_key_syntaxerror OK
"###);
}

/// Ported from `tests/cpython/errors/pep/634/duplicate_attribute_binding_typeerror.py`.
#[test]
fn test_gen_errors_pep_634_duplicate_attribute_binding_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "errors"
# case = "duplicate_attribute_binding_typeerror"
# subject = "match.class_pattern"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.class_pattern: duplicate_attribute_binding_typeerror (errors)."""
pass

_raised = False
try:
    exec("class C:\n __match_args__ = ('a', 'a')\n a = None\nmatch C():\n case C(_p, _q): pass")
except TypeError:
    _raised = True
assert _raised, "duplicate_attribute_binding_typeerror: expected TypeError"
print("duplicate_attribute_binding_typeerror OK")
"###);
    assert_output(&out, r###"duplicate_attribute_binding_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/pep/634/match_args_non_str_entry_typeerror.py`.
#[test]
fn test_gen_errors_pep_634_match_args_non_str_entry_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "errors"
# case = "match_args_non_str_entry_typeerror"
# subject = "match.class_pattern"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.class_pattern: match_args_non_str_entry_typeerror (errors)."""
pass

_raised = False
try:
    exec("class C:\n __match_args__ = (None,)\nmatch C():\n case C(_a): pass")
except TypeError:
    _raised = True
assert _raised, "match_args_non_str_entry_typeerror: expected TypeError"
print("match_args_non_str_entry_typeerror OK")
"###);
    assert_output(&out, r###"match_args_non_str_entry_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/pep/634/match_args_not_tuple_typeerror.py`.
#[test]
fn test_gen_errors_pep_634_match_args_not_tuple_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "errors"
# case = "match_args_not_tuple_typeerror"
# subject = "match.class_pattern"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.class_pattern: match_args_not_tuple_typeerror (errors)."""
pass

_raised = False
try:
    exec("class C:\n __match_args__ = None\nmatch C():\n case C(_a): pass")
except TypeError:
    _raised = True
assert _raised, "match_args_not_tuple_typeerror: expected TypeError"
print("match_args_not_tuple_typeerror OK")
"###);
    assert_output(&out, r###"match_args_not_tuple_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/pep/634/runtime_equal_mapping_keys_valueerror.py`.
#[test]
fn test_gen_errors_pep_634_runtime_equal_mapping_keys_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "errors"
# case = "runtime_equal_mapping_keys_valueerror"
# subject = "match.mapping_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.mapping_pattern: two mapping-pattern keys that compare equal at runtime raise ValueError during matching"""

# Two mapping-pattern keys that compare equal at runtime raise ValueError.
class Keys:
    KEY = "a"


runtime_dup = False
try:
    match {"a": 0, "b": 1}:
        case {Keys.KEY: _y, "a": _z}:
            pass
except ValueError:
    runtime_dup = True
assert runtime_dup is True
print("runtime_equal_mapping_keys_valueerror OK")
"###);
    assert_output(&out, r###"runtime_equal_mapping_keys_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/pep/634/too_many_positional_subpatterns_typeerror.py`.
#[test]
fn test_gen_errors_pep_634_too_many_positional_subpatterns_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "errors"
# case = "too_many_positional_subpatterns_typeerror"
# subject = "match.class_pattern"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.class_pattern: too_many_positional_subpatterns_typeerror (errors)."""
pass

_raised = False
try:
    exec("class C:\n __match_args__ = ()\nmatch C():\n case C(_a): pass")
except TypeError:
    _raised = True
assert _raised, "too_many_positional_subpatterns_typeerror: expected TypeError"
print("too_many_positional_subpatterns_typeerror OK")
"###);
    assert_output(&out, r###"too_many_positional_subpatterns_typeerror OK
"###);
}
