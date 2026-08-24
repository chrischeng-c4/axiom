use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/core/operator_dispatch/add_module_attr_none_operand_raises.py`.
#[test]
fn test_gen_behavior_core_operator_dispatch_add_module_attr_none_operand_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "operator_dispatch"
# dimension = "behavior"
# case = "add_module_attr_none_operand_raises"
# subject = "binary + operand type dispatch"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Binary `+` type-mismatch check must hard-fault identically whether the
None operand is a local variable or reached via module-attribute access.

A local `x = None; x + 'i'` is caught by mamba's static type checker at
compile time. Reading the same None value through a module attribute
(`mod.attr`) defeats that static narrowing, so the mismatch must instead
be caught at runtime by the `+` dispatcher — this regresses a bug (#1938)
where a None operand reached via attribute access silently returned None
instead of raising, unlike the local-variable case.
"""
import types

mod = types.ModuleType("mamba_regression_1938_mod")
mod.attr = None

try:
    result = mod.attr + "i"
    raise AssertionError(f"expected TypeError, got {result!r}")
except TypeError as e:
    assert str(e) == "unsupported operand type(s) for +: 'NoneType' and 'str'", str(e)

print("add_module_attr_none_operand_raises OK")
"###);
    assert_output(&out, r###"add_module_attr_none_operand_raises OK
"###);
}

/// Ported from `tests/cpython/behavior/core/operator_dispatch/add_pending_index_error_not_clobbered.py`.
#[test]
fn test_gen_behavior_core_operator_dispatch_add_pending_index_error_not_clobbered() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "operator_dispatch"
# dimension = "behavior"
# case = "add_pending_index_error_not_clobbered"
# subject = "binary + pending-exception propagation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A `+` left operand whose own evaluation already raised must propagate
that original exception, not a fresh operand-type TypeError from the `+`
dispatcher itself (#1962, uncovered while making #1938's None-operand
check strict: the dispatcher's own pending-exception guard, mirroring the
#1547 mb_value_cmp precedent, is what keeps this case correct).

Indexing the empty tuple `args[0]` raises IndexError before `+` evaluates
the right-hand `args[1]` subscript.
"""
args = ()

try:
    args[0] + args[1]
    raise AssertionError("expected IndexError")
except IndexError:
    pass

print("add_pending_index_error_not_clobbered OK")
"###);
    assert_output(&out, r###"add_pending_index_error_not_clobbered OK
"###);
}

/// Ported from `tests/cpython/behavior/core/operator_dispatch/add_pending_overflow_error_not_clobbered.py`.
#[test]
fn test_gen_behavior_core_operator_dispatch_add_pending_overflow_error_not_clobbered() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "operator_dispatch"
# dimension = "behavior"
# case = "add_pending_overflow_error_not_clobbered"
# subject = "binary + pending-exception propagation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A `+` left operand whose own evaluation already raised must propagate
that original exception, not a fresh operand-type TypeError from the `+`
dispatcher itself (#1962, uncovered while making #1938's None-operand
check strict: the dispatcher's own pending-exception guard, mirroring the
#1547 mb_value_cmp precedent, is what keeps this case correct).

`datetime.timedelta(days=10**10)` overflows during construction, so the
left operand's own evaluation raises OverflowError before `+` evaluates
the right-hand `timedelta(...)` constructor or inspects either result.
"""
import datetime

try:
    datetime.timedelta(days=10**10) + datetime.timedelta(days=10**10)
    raise AssertionError("expected OverflowError")
except OverflowError:
    pass

print("add_pending_overflow_error_not_clobbered OK")
"###);
    assert_output(&out, r###"add_pending_overflow_error_not_clobbered OK
"###);
}

/// Ported from `tests/cpython/behavior/core/operator_dispatch/add_unannotated_param_none_operand_raises.py`.
#[test]
fn test_gen_behavior_core_operator_dispatch_add_unannotated_param_none_operand_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "operator_dispatch"
# dimension = "behavior"
# case = "add_unannotated_param_none_operand_raises"
# subject = "binary + operand type dispatch"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Binary `+` type-mismatch check must hard-fault when a None operand
arrives through an unannotated (Any-typed) function parameter, matching
CPython and mamba's own local-variable behavior (#1938).

An unannotated parameter is statically Any, so mamba cannot narrow its
type at compile time the way it can for a direct `x = None` assignment;
the None-vs-str mismatch must be caught at runtime by the `+` dispatcher
instead.
"""


def combine(value):
    return value + "i"


try:
    result = combine(None)
    raise AssertionError(f"expected TypeError, got {result!r}")
except TypeError as e:
    assert str(e) == "unsupported operand type(s) for +: 'NoneType' and 'str'", str(e)

print("add_unannotated_param_none_operand_raises OK")
"###);
    assert_output(&out, r###"add_unannotated_param_none_operand_raises OK
"###);
}

/// Ported from `tests/cpython/behavior/core/operator_dispatch/floordiv_pending_index_error_not_clobbered.py`.
#[test]
fn test_gen_behavior_core_operator_dispatch_floordiv_pending_index_error_not_clobbered() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "operator_dispatch"
# dimension = "behavior"
# case = "floordiv_pending_index_error_not_clobbered"
# subject = "binary // pending-exception propagation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A `//` right operand whose own evaluation already raised must propagate
that original exception, not a fresh operand-type TypeError from the
datetime-arithmetic operand-mismatch check shared by `+ - * / % //`
(#1962, same pending-exception guard shape as #1547 mb_value_cmp / #1938
mb_add — here it guards `raise_datetime_op_type_error`, which fires
whenever one operand is a `datetime.*` instance regardless of whether the
other operand is a real value or the `None` sentinel a raise leaves
behind).

The left operand `datetime.timedelta(days=1)` evaluates successfully;
indexing the empty tuple `args[0]` then raises IndexError while
evaluating the right-hand operand.
"""
import datetime

args = ()

try:
    datetime.timedelta(days=1) // args[0]
    raise AssertionError("expected IndexError")
except IndexError:
    pass

print("floordiv_pending_index_error_not_clobbered OK")
"###);
    assert_output(&out, r###"floordiv_pending_index_error_not_clobbered OK
"###);
}

/// Ported from `tests/cpython/behavior/core/operator_dispatch/gt_instance_pending_index_error_not_clobbered.py`.
#[test]
fn test_gen_behavior_core_operator_dispatch_gt_instance_pending_index_error_not_clobbered() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "operator_dispatch"
# dimension = "behavior"
# case = "gt_instance_pending_index_error_not_clobbered"
# subject = "binary > pending-exception propagation (Instance operand)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A `>` right operand whose own evaluation already raised must propagate
that original exception, not a fresh unorderable-types TypeError from
`unsupported_ordering_bool` (#1962, same pending-exception guard shape as
#1547 mb_value_cmp / #1938 mb_add). When the left operand is a real
`Instance` without `__gt__` (and the right operand is the `None` sentinel a
raise left behind), `mb_gt` reaches `unsupported_ordering_bool` directly
instead of composing through `mb_lt`/`values_lt_fallback`, so this is a
distinct raise site from the plain `<` case.

The left operand `Plain()` evaluates successfully; indexing the empty tuple
`args[0]` then raises IndexError while evaluating the right-hand operand.
"""


class Plain:
    pass


args = ()

try:
    Plain() > args[0]
    raise AssertionError("expected IndexError")
except IndexError:
    pass

print("gt_instance_pending_index_error_not_clobbered OK")
"###);
    assert_output(&out, r###"gt_instance_pending_index_error_not_clobbered OK
"###);
}

/// Ported from `tests/cpython/behavior/core/operator_dispatch/lt_pending_index_error_not_clobbered.py`.
#[test]
fn test_gen_behavior_core_operator_dispatch_lt_pending_index_error_not_clobbered() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "operator_dispatch"
# dimension = "behavior"
# case = "lt_pending_index_error_not_clobbered"
# subject = "binary < pending-exception propagation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A `<` right operand whose own evaluation already raised must propagate
that original exception, not a fresh unorderable-types TypeError from the
comparison dispatcher's final fallback (#1962, same pending-exception guard
shape as #1547 mb_value_cmp / #1938 mb_add — here it guards
`values_lt_fallback`, the shared tail `mb_lt` falls through to and that
`mb_gt`/`mb_le`/`mb_ge` also compose through via `mb_lt(b, a)`).

The left operand `5` evaluates successfully; indexing the empty tuple
`args[0]` then raises IndexError while evaluating the right-hand operand.
"""
args = ()

try:
    5 < args[0]
    raise AssertionError("expected IndexError")
except IndexError:
    pass

print("lt_pending_index_error_not_clobbered OK")
"###);
    assert_output(&out, r###"lt_pending_index_error_not_clobbered OK
"###);
}

/// Ported from `tests/cpython/behavior/core/operator_dispatch/mod_pending_index_error_not_clobbered.py`.
#[test]
fn test_gen_behavior_core_operator_dispatch_mod_pending_index_error_not_clobbered() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "operator_dispatch"
# dimension = "behavior"
# case = "mod_pending_index_error_not_clobbered"
# subject = "binary % pending-exception propagation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A `%` right operand whose own evaluation already raised must propagate
that original exception, not a fresh operand-type TypeError from the
datetime-arithmetic operand-mismatch check shared by `+ - * / % //`
(#1962, same pending-exception guard shape as #1547 mb_value_cmp / #1938
mb_add — here it guards `raise_datetime_op_type_error`, which fires
whenever one operand is a `datetime.*` instance regardless of whether the
other operand is a real value or the `None` sentinel a raise leaves
behind).

The left operand `datetime.timedelta(days=1)` evaluates successfully;
indexing the empty tuple `args[0]` then raises IndexError while
evaluating the right-hand operand.
"""
import datetime

args = ()

try:
    datetime.timedelta(days=1) % args[0]
    raise AssertionError("expected IndexError")
except IndexError:
    pass

print("mod_pending_index_error_not_clobbered OK")
"###);
    assert_output(&out, r###"mod_pending_index_error_not_clobbered OK
"###);
}

/// Ported from `tests/cpython/behavior/core/operator_dispatch/mul_pending_index_error_not_clobbered.py`.
#[test]
fn test_gen_behavior_core_operator_dispatch_mul_pending_index_error_not_clobbered() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "operator_dispatch"
# dimension = "behavior"
# case = "mul_pending_index_error_not_clobbered"
# subject = "binary * pending-exception propagation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A `*` right operand whose own evaluation already raised must propagate
that original exception, not a fresh "can't multiply sequence" TypeError
from the `*` dispatcher's sequence-repeat tail (#1962, same pending-
exception guard shape as #1547 mb_value_cmp / #1938 mb_add — here it
guards `mb_mul`'s non-int-repeat-count raise instead).

Indexing the empty tuple `args[0]` raises IndexError while evaluating the
right-hand operand of `[1, 2, 3] * args[0]`, after the list literal on the
left has already evaluated successfully.
"""
args = ()

try:
    [1, 2, 3] * args[0]
    raise AssertionError("expected IndexError")
except IndexError:
    pass

print("mul_pending_index_error_not_clobbered OK")
"###);
    assert_output(&out, r###"mul_pending_index_error_not_clobbered OK
"###);
}

/// Ported from `tests/cpython/behavior/core/operator_dispatch/pow_pending_index_error_not_clobbered.py`.
#[test]
fn test_gen_behavior_core_operator_dispatch_pow_pending_index_error_not_clobbered() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "operator_dispatch"
# dimension = "behavior"
# case = "pow_pending_index_error_not_clobbered"
# subject = "binary ** pending-exception propagation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A `**` left operand whose own evaluation already raised must propagate
that original exception, not a fresh operand-type TypeError from the `**`
dispatcher's final catch-all tail (#1962, same pending-exception guard
shape as #1547 mb_value_cmp / #1938 mb_add — here it guards `mb_pow`'s
unsupported-operand-pair raise instead).

Indexing the empty tuple `args[0]` raises IndexError before `**` evaluates
the right-hand `args[1]` subscript.
"""
args = ()

try:
    args[0] ** args[1]
    raise AssertionError("expected IndexError")
except IndexError:
    pass

print("pow_pending_index_error_not_clobbered OK")
"###);
    assert_output(&out, r###"pow_pending_index_error_not_clobbered OK
"###);
}

/// Ported from `tests/cpython/behavior/core/operator_dispatch/sub_pending_index_error_not_clobbered.py`.
#[test]
fn test_gen_behavior_core_operator_dispatch_sub_pending_index_error_not_clobbered() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "operator_dispatch"
# dimension = "behavior"
# case = "sub_pending_index_error_not_clobbered"
# subject = "binary - pending-exception propagation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A `-` right operand whose own evaluation already raised must propagate
that original exception, not a fresh operand-type TypeError from the
datetime-arithmetic operand-mismatch check shared by `+ - * / % //`
(#1962, same pending-exception guard shape as #1547 mb_value_cmp / #1938
mb_add — here it guards `raise_datetime_op_type_error`, which fires
whenever one operand is a `datetime.*` instance regardless of whether the
other operand is a real value or the `None` sentinel a raise leaves
behind, so it is not masked by `mb_sub`'s own None-operand leniency).

The left operand `datetime.timedelta(days=1)` evaluates successfully;
indexing the empty tuple `args[0]` then raises IndexError while
evaluating the right-hand operand.
"""
import datetime

args = ()

try:
    datetime.timedelta(days=1) - args[0]
    raise AssertionError("expected IndexError")
except IndexError:
    pass

print("sub_pending_index_error_not_clobbered OK")
"###);
    assert_output(&out, r###"sub_pending_index_error_not_clobbered OK
"###);
}

/// Ported from `tests/cpython/behavior/core/operator_dispatch/truediv_pending_index_error_not_clobbered.py`.
#[test]
fn test_gen_behavior_core_operator_dispatch_truediv_pending_index_error_not_clobbered() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "operator_dispatch"
# dimension = "behavior"
# case = "truediv_pending_index_error_not_clobbered"
# subject = "binary / pending-exception propagation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A `/` right operand whose own evaluation already raised must propagate
that original exception, not a fresh operand-type TypeError from the
datetime-arithmetic operand-mismatch check shared by `+ - * / % //`
(#1962, same pending-exception guard shape as #1547 mb_value_cmp / #1938
mb_add — here it guards `raise_datetime_op_type_error`, which fires
whenever one operand is a `datetime.*` instance regardless of whether the
other operand is a real value or the `None` sentinel a raise leaves
behind).

The left operand `datetime.timedelta(days=1)` evaluates successfully;
indexing the empty tuple `args[0]` then raises IndexError while
evaluating the right-hand operand.
"""
import datetime

args = ()

try:
    datetime.timedelta(days=1) / args[0]
    raise AssertionError("expected IndexError")
except IndexError:
    pass

print("truediv_pending_index_error_not_clobbered OK")
"###);
    assert_output(&out, r###"truediv_pending_index_error_not_clobbered OK
"###);
}
