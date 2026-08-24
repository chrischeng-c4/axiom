use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/functools/cmp_to_key_result_not_hashable_raises.py`.
#[test]
fn test_gen_errors_std_libs_functools_cmp_to_key_result_not_hashable_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "errors"
# case = "cmp_to_key_result_not_hashable_raises"
# subject = "functools.cmp_to_key"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.cmp_to_key: cmp_to_key_result_not_hashable_raises (errors)."""
import functools

_raised = False
try:
    hash(functools.cmp_to_key(lambda x, y: 0)(1))
except TypeError:
    _raised = True
assert _raised, "cmp_to_key_result_not_hashable_raises: expected TypeError"
print("cmp_to_key_result_not_hashable_raises OK")
"###);
    assert_output(&out, r###"cmp_to_key_result_not_hashable_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/functools/lru_cache_bad_maxsize_raises.py`.
#[test]
fn test_gen_errors_std_libs_functools_lru_cache_bad_maxsize_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "errors"
# case = "lru_cache_bad_maxsize_raises"
# subject = "functools.lru_cache"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.lru_cache: lru_cache_bad_maxsize_raises (errors)."""
import functools

_raised = False
try:
    functools.lru_cache(maxsize="all")(lambda x: x)
except TypeError:
    _raised = True
assert _raised, "lru_cache_bad_maxsize_raises: expected TypeError"
print("lru_cache_bad_maxsize_raises OK")
"###);
    assert_output(&out, r###"lru_cache_bad_maxsize_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/functools/partial_no_args_raises.py`.
#[test]
fn test_gen_errors_std_libs_functools_partial_no_args_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "errors"
# case = "partial_no_args_raises"
# subject = "functools.partial"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.partial: partial_no_args_raises (errors)."""
import functools

_raised = False
try:
    functools.partial()
except TypeError:
    _raised = True
assert _raised, "partial_no_args_raises: expected TypeError"
print("partial_no_args_raises OK")
"###);
    assert_output(&out, r###"partial_no_args_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/functools/partial_non_callable_raises.py`.
#[test]
fn test_gen_errors_std_libs_functools_partial_non_callable_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "errors"
# case = "partial_non_callable_raises"
# subject = "functools.partial"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.partial: partial_non_callable_raises (errors)."""
import functools

_raised = False
try:
    functools.partial(42, 1)
except TypeError:
    _raised = True
assert _raised, "partial_non_callable_raises: expected TypeError"
print("partial_non_callable_raises OK")
"###);
    assert_output(&out, r###"partial_non_callable_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/functools/partialmethod_non_callable_raises.py`.
#[test]
fn test_gen_errors_std_libs_functools_partialmethod_non_callable_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "errors"
# case = "partialmethod_non_callable_raises"
# subject = "functools.partialmethod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.partialmethod: partialmethod_non_callable_raises (errors)."""
import functools

_raised = False
try:
    type("Bad", (), {"m": functools.partialmethod(None, 1)})
except TypeError:
    _raised = True
assert _raised, "partialmethod_non_callable_raises: expected TypeError"
print("partialmethod_non_callable_raises OK")
"###);
    assert_output(&out, r###"partialmethod_non_callable_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/functools/reduce_empty_no_initial_raises.py`.
#[test]
fn test_gen_errors_std_libs_functools_reduce_empty_no_initial_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "errors"
# case = "reduce_empty_no_initial_raises"
# subject = "functools.reduce"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.reduce: reduce_empty_no_initial_raises (errors)."""
import functools

_raised = False
try:
    functools.reduce(lambda a, b: a + b, [])
except TypeError:
    _raised = True
assert _raised, "reduce_empty_no_initial_raises: expected TypeError"
print("reduce_empty_no_initial_raises OK")
"###);
    assert_output(&out, r###"reduce_empty_no_initial_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/functools/reduce_non_iterable_raises.py`.
#[test]
fn test_gen_errors_std_libs_functools_reduce_non_iterable_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "errors"
# case = "reduce_non_iterable_raises"
# subject = "functools.reduce"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.reduce: reduce_non_iterable_raises (errors)."""
import functools

_raised = False
try:
    functools.reduce(lambda a, b: a + b, 123)
except TypeError:
    _raised = True
assert _raised, "reduce_non_iterable_raises: expected TypeError"
print("reduce_non_iterable_raises OK")
"###);
    assert_output(&out, r###"reduce_non_iterable_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/functools/singledispatch_no_positional_arg_raises.py`.
#[test]
fn test_gen_errors_std_libs_functools_singledispatch_no_positional_arg_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "errors"
# case = "singledispatch_no_positional_arg_raises"
# subject = "functools.singledispatch"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.singledispatch: singledispatch_no_positional_arg_raises (errors)."""
import functools

_raised = False
try:
    functools.singledispatch(lambda *a: None)()
except TypeError:
    _raised = True
assert _raised, "singledispatch_no_positional_arg_raises: expected TypeError"
print("singledispatch_no_positional_arg_raises OK")
"###);
    assert_output(&out, r###"singledispatch_no_positional_arg_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/functools/total_ordering_no_op_raises.py`.
#[test]
fn test_gen_errors_std_libs_functools_total_ordering_no_op_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "errors"
# case = "total_ordering_no_op_raises"
# subject = "functools.total_ordering"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.total_ordering: total_ordering_no_op_raises (errors)."""
import functools

_raised = False
try:
    functools.total_ordering(type("E", (), {}))
except ValueError:
    _raised = True
assert _raised, "total_ordering_no_op_raises: expected ValueError"
print("total_ordering_no_op_raises OK")
"###);
    assert_output(&out, r###"total_ordering_no_op_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/functools/update_wrapper_missing_updated_attr_raises.py`.
#[test]
fn test_gen_errors_std_libs_functools_update_wrapper_missing_updated_attr_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "errors"
# case = "update_wrapper_missing_updated_attr_raises"
# subject = "functools.update_wrapper"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.update_wrapper: update_wrapper_missing_updated_attr_raises (errors)."""
import functools

_raised = False
try:
    functools.update_wrapper(lambda: 0, lambda: 0, assigned=("attr",), updated=("missing_d",))
except AttributeError:
    _raised = True
assert _raised, "update_wrapper_missing_updated_attr_raises: expected AttributeError"
print("update_wrapper_missing_updated_attr_raises OK")
"###);
    assert_output(&out, r###"update_wrapper_missing_updated_attr_raises OK
"###);
}
