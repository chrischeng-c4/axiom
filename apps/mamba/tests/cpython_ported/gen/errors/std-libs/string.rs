use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/string/formatter_bad_index_raises.py`.
#[test]
fn test_gen_errors_std_libs_string_formatter_bad_index_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "errors"
# case = "formatter_bad_index_raises"
# subject = "string.Formatter"
# kind = "mechanical"
# xfail = "string.Formatter is a silent dict-stub on mamba; .vformat() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.Formatter: formatter_bad_index_raises (errors)."""
import string

_raised = False
try:
    string.Formatter().vformat('{5}', [1, 2], {})
except IndexError:
    _raised = True
assert _raised, "formatter_bad_index_raises: expected IndexError"
print("formatter_bad_index_raises OK")
"###);
    assert_output(&out, r###"formatter_bad_index_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/string/formatter_bad_spec_raises.py`.
#[test]
fn test_gen_errors_std_libs_string_formatter_bad_spec_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "errors"
# case = "formatter_bad_spec_raises"
# subject = "string.Formatter"
# kind = "mechanical"
# xfail = "string.Formatter is a silent dict-stub on mamba; .format() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.Formatter: formatter_bad_spec_raises (errors)."""
import string

_raised = False
try:
    string.Formatter().format('{:Q}', 1)
except ValueError:
    _raised = True
assert _raised, "formatter_bad_spec_raises: expected ValueError"
print("formatter_bad_spec_raises OK")
"###);
    assert_output(&out, r###"formatter_bad_spec_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/string/formatter_missing_kw_raises.py`.
#[test]
fn test_gen_errors_std_libs_string_formatter_missing_kw_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "errors"
# case = "formatter_missing_kw_raises"
# subject = "string.Formatter"
# kind = "mechanical"
# xfail = "string.Formatter is a silent dict-stub on mamba; .vformat() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.Formatter: formatter_missing_kw_raises (errors)."""
import string

_raised = False
try:
    string.Formatter().vformat('{name}', [], {})
except KeyError:
    _raised = True
assert _raised, "formatter_missing_kw_raises: expected KeyError"
print("formatter_missing_kw_raises OK")
"###);
    assert_output(&out, r###"formatter_missing_kw_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/string/formatter_unknown_conversion_raises.py`.
#[test]
fn test_gen_errors_std_libs_string_formatter_unknown_conversion_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "errors"
# case = "formatter_unknown_conversion_raises"
# subject = "string.Formatter"
# kind = "mechanical"
# xfail = "string.Formatter is a silent dict-stub on mamba; .format() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.Formatter: formatter_unknown_conversion_raises (errors)."""
import string

_raised = False
try:
    string.Formatter().format('{0!h}', 'test')
except ValueError:
    _raised = True
assert _raised, "formatter_unknown_conversion_raises: expected ValueError"
print("formatter_unknown_conversion_raises OK")
"###);
    assert_output(&out, r###"formatter_unknown_conversion_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/string/template_bad_identifier_raises.py`.
#[test]
fn test_gen_errors_std_libs_string_template_bad_identifier_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "errors"
# case = "template_bad_identifier_raises"
# subject = "string.Template"
# kind = "mechanical"
# xfail = "string.Template is a silent dict-stub on mamba; .substitute() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.Template: template_bad_identifier_raises (errors)."""
import string

_raised = False
try:
    string.Template('$0_bad').substitute({'0_bad': 'value'})
except ValueError:
    _raised = True
assert _raised, "template_bad_identifier_raises: expected ValueError"
print("template_bad_identifier_raises OK")
"###);
    assert_output(&out, r###"template_bad_identifier_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/string/template_missing_key_raises.py`.
#[test]
fn test_gen_errors_std_libs_string_template_missing_key_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "errors"
# case = "template_missing_key_raises"
# subject = "string.Template"
# kind = "mechanical"
# xfail = "string.Template is a silent dict-stub on mamba; .substitute() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.Template: template_missing_key_raises (errors)."""
import string

_raised = False
try:
    string.Template('$missing').substitute({})
except KeyError:
    _raised = True
assert _raised, "template_missing_key_raises: expected KeyError"
print("template_missing_key_raises OK")
"###);
    assert_output(&out, r###"template_missing_key_raises OK
"###);
}
