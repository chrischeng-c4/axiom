use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/heapq/heapify_mixed_types_raises.py`.
#[test]
fn test_gen_errors_std_libs_heapq_heapify_mixed_types_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "errors"
# case = "heapify_mixed_types_raises"
# subject = "heapq.heapify"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heapify: heapify_mixed_types_raises (errors)."""
import heapq

_raised = False
try:
    heapq.heapify([1, "two", 3])
except TypeError:
    _raised = True
assert _raised, "heapify_mixed_types_raises: expected TypeError"
print("heapify_mixed_types_raises OK")
"###);
    assert_output(&out, r###"heapify_mixed_types_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/heapq/heappop_empty_raises.py`.
#[test]
fn test_gen_errors_std_libs_heapq_heappop_empty_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "errors"
# case = "heappop_empty_raises"
# subject = "heapq.heappop"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heappop: heappop_empty_raises (errors)."""
import heapq

_raised = False
try:
    heapq.heappop([])
except IndexError:
    _raised = True
assert _raised, "heappop_empty_raises: expected IndexError"
print("heappop_empty_raises OK")
"###);
    assert_output(&out, r###"heappop_empty_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/heapq/heappushpop_empty_no_raise.py`.
#[test]
fn test_gen_errors_std_libs_heapq_heappushpop_empty_no_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "errors"
# case = "heappushpop_empty_no_raise"
# subject = "heapq.heappushpop"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heappushpop: heappushpop on an empty heap does NOT raise: with no root to compare, the pushed item is returned unchanged"""
import heapq

# Unlike heappop/heapreplace, heappushpop on an empty heap is well-defined:
# the pushed item is returned and the heap stays empty.
_h = []
_out = heapq.heappushpop(_h, 42)
assert _out == 42, f"heappushpop([], 42) = {_out!r}"
assert _h == [], f"heap stays empty = {_h!r}"
print("heappushpop_empty_no_raise OK")
"###);
    assert_output(&out, r###"heappushpop_empty_no_raise OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/heapq/heapreplace_empty_raises.py`.
#[test]
fn test_gen_errors_std_libs_heapq_heapreplace_empty_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "errors"
# case = "heapreplace_empty_raises"
# subject = "heapq.heapreplace"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heapreplace: heapreplace_empty_raises (errors)."""
import heapq

_raised = False
try:
    heapq.heapreplace([], 1)
except IndexError:
    _raised = True
assert _raised, "heapreplace_empty_raises: expected IndexError"
print("heapreplace_empty_raises OK")
"###);
    assert_output(&out, r###"heapreplace_empty_raises OK
"###);
}
