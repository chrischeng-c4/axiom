use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/textwrap/fill_zero_width_raises.py`.
#[test]
fn test_gen_errors_std_libs_textwrap_fill_zero_width_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "errors"
# case = "fill_zero_width_raises"
# subject = "textwrap.fill"
# kind = "mechanical"
# xfail = "textwrap.fill is a silent stub under mamba — width<=0 does not raise (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.fill: fill_zero_width_raises (errors)."""
import textwrap

_raised = False
try:
    textwrap.fill("hello", width=0)
except ValueError:
    _raised = True
assert _raised, "fill_zero_width_raises: expected ValueError"
print("fill_zero_width_raises OK")
"###);
    assert_output(&out, r###"fill_zero_width_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/textwrap/shorten_placeholder_over_width_raises.py`.
#[test]
fn test_gen_errors_std_libs_textwrap_shorten_placeholder_over_width_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "errors"
# case = "shorten_placeholder_over_width_raises"
# subject = "textwrap.shorten"
# kind = "mechanical"
# xfail = "textwrap.shorten is a silent stub under mamba — placeholder-too-large does not raise (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.shorten: shorten_placeholder_over_width_raises (errors)."""
import textwrap

_raised = False
try:
    textwrap.shorten("x" * 20, width=8, placeholder="(.......)")
except ValueError:
    _raised = True
assert _raised, "shorten_placeholder_over_width_raises: expected ValueError"
print("shorten_placeholder_over_width_raises OK")
"###);
    assert_output(&out, r###"shorten_placeholder_over_width_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/textwrap/shorten_width_too_small_raises.py`.
#[test]
fn test_gen_errors_std_libs_textwrap_shorten_width_too_small_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "errors"
# case = "shorten_width_too_small_raises"
# subject = "textwrap.shorten"
# kind = "mechanical"
# xfail = "textwrap.shorten is a silent stub under mamba — placeholder-too-large does not raise (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.shorten: shorten_width_too_small_raises (errors)."""
import textwrap

_raised = False
try:
    textwrap.shorten("hello world", width=2, placeholder="[...]")
except ValueError:
    _raised = True
assert _raised, "shorten_width_too_small_raises: expected ValueError"
print("shorten_width_too_small_raises OK")
"###);
    assert_output(&out, r###"shorten_width_too_small_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/textwrap/wrap_negative_width_raises.py`.
#[test]
fn test_gen_errors_std_libs_textwrap_wrap_negative_width_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "errors"
# case = "wrap_negative_width_raises"
# subject = "textwrap.wrap"
# kind = "mechanical"
# xfail = "textwrap.wrap is a silent stub under mamba — width<=0 does not raise (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.wrap: wrap_negative_width_raises (errors)."""
import textwrap

_raised = False
try:
    textwrap.wrap("hello", width=-5)
except ValueError:
    _raised = True
assert _raised, "wrap_negative_width_raises: expected ValueError"
print("wrap_negative_width_raises OK")
"###);
    assert_output(&out, r###"wrap_negative_width_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/textwrap/wrap_zero_width_raises.py`.
#[test]
fn test_gen_errors_std_libs_textwrap_wrap_zero_width_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "errors"
# case = "wrap_zero_width_raises"
# subject = "textwrap.wrap"
# kind = "mechanical"
# xfail = "textwrap.wrap is a silent stub under mamba — width<=0 does not raise (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.wrap: wrap_zero_width_raises (errors)."""
import textwrap

_raised = False
try:
    textwrap.wrap("hello", width=0)
except ValueError:
    _raised = True
assert _raised, "wrap_zero_width_raises: expected ValueError"
print("wrap_zero_width_raises OK")
"###);
    assert_output(&out, r###"wrap_zero_width_raises OK
"###);
}
