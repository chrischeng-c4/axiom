use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/pep/501/honors_format_spec.py`.
#[test]
fn test_gen_behavior_pep_501_honors_format_spec() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "501"
# dimension = "behavior"
# case = "honors_format_spec"
# subject = "fstring.debug_equals"
# kind = "semantic"
# xfail = "mamba strips the `name = ` prefix from PEP 501 debug f-strings (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.debug_equals: a format spec after the debug '=' applies to the value only: f"{x = :04d}" -> 'x = 0042'"""

x = 42
assert f"{x = :04d}" == "x = 0042", repr(f"{x = :04d}")
print("honors_format_spec OK")
"###);
    assert_output(&out, r###"honors_format_spec OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/501/multiple_debug_fields.py`.
#[test]
fn test_gen_behavior_pep_501_multiple_debug_fields() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "501"
# dimension = "behavior"
# case = "multiple_debug_fields"
# subject = "fstring.debug_equals"
# kind = "semantic"
# xfail = "mamba strips the `name = ` prefix from PEP 501 debug f-strings (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.debug_equals: two `{expr = }` fields in one f-string each render their own 'name = value' segment -> "x = 42, y = 'hi'" """

x = 42
y = "hi"
assert f"{x = }, {y = }" == "x = 42, y = 'hi'", repr(f"{x = }, {y = }")
print("multiple_debug_fields OK")
"###);
    assert_output(&out, r###"multiple_debug_fields OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/501/renders_expression_source.py`.
#[test]
fn test_gen_behavior_pep_501_renders_expression_source() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "501"
# dimension = "behavior"
# case = "renders_expression_source"
# subject = "fstring.debug_equals"
# kind = "semantic"
# xfail = "mamba strips the `expr = ` prefix from PEP 501 debug f-strings (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.debug_equals: f"{x + 1 = }" echoes the full expression source text before the '=' -> 'x + 1 = 4'"""

x = 3
assert f"{x + 1 = }" == "x + 1 = 4", repr(f"{x + 1 = }")
print("renders_expression_source OK")
"###);
    assert_output(&out, r###"renders_expression_source OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/501/renders_name_equals_value.py`.
#[test]
fn test_gen_behavior_pep_501_renders_name_equals_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "501"
# dimension = "behavior"
# case = "renders_name_equals_value"
# subject = "fstring.debug_equals"
# kind = "semantic"
# xfail = "mamba strips the `name = ` prefix from PEP 501 debug f-strings, rendering just the value (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.debug_equals: f"{x = }" renders the expression source, an '=', and the value -> 'x = 10'"""

x = 10
assert f"{x = }" == "x = 10", repr(f"{x = }")
print("renders_name_equals_value OK")
"###);
    assert_output(&out, r###"renders_name_equals_value OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/501/repr_conversion_applies.py`.
#[test]
fn test_gen_behavior_pep_501_repr_conversion_applies() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "501"
# dimension = "behavior"
# case = "repr_conversion_applies"
# subject = "fstring.debug_equals"
# kind = "semantic"
# xfail = "mamba strips the `name = ` prefix from PEP 501 debug f-strings (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.debug_equals: an explicit !r conversion after the debug '=' reprs the value: f"{x = !r}" -> "x = 42" with the int repr"""

x = 42
# !r reprs the value (int repr is plain '42'); the 's' below confirms the
# conversion is honored, not the implicit default that the bare '=' would use.
assert f"{x = !r}" == "x = 42", repr(f"{x = !r}")
s = "ab"
assert f"{s = !r}" == "s = 'ab'", repr(f"{s = !r}")
print("repr_conversion_applies OK")
"###);
    assert_output(&out, r###"repr_conversion_applies OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/501/whitespace_around_equals_preserved.py`.
#[test]
fn test_gen_behavior_pep_501_whitespace_around_equals_preserved() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "501"
# dimension = "behavior"
# case = "whitespace_around_equals_preserved"
# subject = "fstring.debug_equals"
# kind = "semantic"
# xfail = "mamba strips the `name = ` prefix from PEP 501 debug f-strings (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.debug_equals: leading/trailing inner whitespace is preserved verbatim: f"{x= }" -> 'x= 10' and f"{x =}" -> 'x =10'"""

x = 10
# The text between the expression and the '=' is echoed verbatim; only one
# trailing space is implied by the default conversion when none is given.
assert f"{x= }" == "x= 10", repr(f"{x= }")
assert f"{x =}" == "x =10", repr(f"{x =}")
print("whitespace_around_equals_preserved OK")
"###);
    assert_output(&out, r###"whitespace_around_equals_preserved OK
"###);
}
