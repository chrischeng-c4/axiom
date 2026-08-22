use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/pep/positional_only/kw_only_required_by_keyword.py`.
#[test]
fn test_gen_behavior_pep_positional_only_kw_only_required_by_keyword() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "positional_only"
# dimension = "behavior"
# case = "kw_only_required_by_keyword"
# subject = "*"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""*: a keyword-only parameter (after `*`) is bound when passed by keyword: def f(*, n, m): return n * m; f(n=3, m=4) == 12"""

# Rule: parameters after a bare `*` are keyword-only and bind when supplied by
# keyword.
def _kw_only(*, n: int, m: int) -> int:
    return n * m

assert _kw_only(n=3, m=4) == 12, _kw_only(n=3, m=4)

print("kw_only_required_by_keyword OK")
"###);
    assert_output(&out, r###"kw_only_required_by_keyword OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/positional_only/kwargs_shadows_posonly_name.py`.
#[test]
fn test_gen_behavior_pep_positional_only_kwargs_shadows_posonly_name() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "positional_only"
# dimension = "behavior"
# case = "kwargs_shadows_posonly_name"
# subject = "/"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""/: a keyword colliding with a positional-only param name lands in **kwargs and does not shadow it: def f(a, /, **kw); f(1, a=999) binds a==1 and kw=={'a': 999}"""

# Rule: a keyword whose name matches a positional-only param does NOT conflict;
# it is captured by **kwargs instead of binding the positional-only parameter.
def _shadowed(a: int, /, **kwargs) -> dict:
    return {"a": a, "kwargs": kwargs}

_r = _shadowed(1, a=999)
assert _r["a"] == 1, _r["a"]
assert _r["kwargs"] == {"a": 999}, _r["kwargs"]

print("kwargs_shadows_posonly_name OK")
"###);
    assert_output(&out, r###"kwargs_shadows_posonly_name OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/positional_only/mixed_defaults_resolution.py`.
#[test]
fn test_gen_behavior_pep_positional_only_mixed_defaults_resolution() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "positional_only"
# dimension = "behavior"
# case = "mixed_defaults_resolution"
# subject = "/"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""/: a mixed pos-only/regular/kw-only signature with defaults resolves each override correctly: def f(po, /, reg=10, *, ko=100) yields 111 / 103 / 16 / 6 across the override combinations"""

# Rule: defaults across pos-only / regular / kw-only resolve independently as
# each override combination is applied.
def _complex(po: int, /, reg: int = 10, *, ko: int = 100) -> int:
    return po + reg + ko

assert _complex(1) == 111, _complex(1)
assert _complex(1, 2) == 103, _complex(1, 2)
assert _complex(1, ko=5) == 16, _complex(1, ko=5)
assert _complex(1, 2, ko=3) == 6, _complex(1, 2, ko=3)

print("mixed_defaults_resolution OK")
"###);
    assert_output(&out, r###"mixed_defaults_resolution OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/positional_only/posonly_names_in_co_varnames.py`.
#[test]
fn test_gen_behavior_pep_positional_only_posonly_names_in_co_varnames() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "positional_only"
# dimension = "behavior"
# case = "posonly_names_in_co_varnames"
# subject = "/"
# kind = "semantic"
# xfail = "mamba function introspection returns None for fn.__code__, so co_varnames is unreadable (project_mamba_function_machinery_silent_divergences #8)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""/: positional-only parameter names still appear in __code__.co_varnames"""

# Rule: positional-only names are real locals, so they remain visible in the
# code object's co_varnames even though they cannot be named at the call site.
def _fn(a: int, b: int, /) -> int:
    return a + b

assert "a" in _fn.__code__.co_varnames, _fn.__code__.co_varnames
assert "b" in _fn.__code__.co_varnames, _fn.__code__.co_varnames

print("posonly_names_in_co_varnames OK")
"###);
    assert_output(&out, r###"posonly_names_in_co_varnames OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/positional_only/posonly_positional_call_returns_value.py`.
#[test]
fn test_gen_behavior_pep_positional_only_posonly_positional_call_returns_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "positional_only"
# dimension = "behavior"
# case = "posonly_positional_call_returns_value"
# subject = "/"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""/: a positional-only function called positionally returns the computed value: def f(a, b, /): return a + b; f(1, 2) == 3"""

# Rule: positional-only params are bound by position and the call returns the
# computed value.
def _fn(a: int, b: int, /) -> int:
    return a + b

assert _fn(1, 2) == 3, _fn(1, 2)

print("posonly_positional_call_returns_value OK")
"###);
    assert_output(&out, r###"posonly_positional_call_returns_value OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/positional_only/regular_param_positional_or_keyword.py`.
#[test]
fn test_gen_behavior_pep_positional_only_regular_param_positional_or_keyword() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "positional_only"
# dimension = "behavior"
# case = "regular_param_positional_or_keyword"
# subject = "/"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""/: a regular (non-pos-only, non-kw-only) parameter can be passed positionally, by keyword, or mixed, all yielding the same result"""

# Rule: parameters with no `/` before them and no `*` are ordinary
# positional-or-keyword params — every call style yields the same result.
def _regular(x: int, y: int) -> int:
    return x + y

assert _regular(3, 4) == 7, "positional"
assert _regular(x=3, y=4) == 7, "keyword"
assert _regular(3, y=4) == 7, "mixed"

print("regular_param_positional_or_keyword OK")
"###);
    assert_output(&out, r###"regular_param_positional_or_keyword OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/positional_only/signature_renders_slash.py`.
#[test]
fn test_gen_behavior_pep_positional_only_signature_renders_slash() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "positional_only"
# dimension = "behavior"
# case = "signature_renders_slash"
# subject = "/"
# kind = "semantic"
# xfail = "mamba function introspection returns None for fn.__code__, so inspect.signature does not reflect the positional-only `/` (project_mamba_function_machinery_silent_divergences #8)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""/: inspect.signature renders the positional-only `/` separator: str(inspect.signature(f)) contains '/'"""
import inspect

# Rule: inspect.signature surfaces the positional-only marker as a `/` in the
# rendered signature string.
def _fn(a: int, b: int, /) -> int:
    return a + b

_sig = str(inspect.signature(_fn))
assert "/" in _sig, _sig

print("signature_renders_slash OK")
"###);
    assert_output(&out, r###"signature_renders_slash OK
"###);
}
