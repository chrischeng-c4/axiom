use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/_contextvars/contextvar_get_accepts_object_default.py`.
#[test]
fn test_gen_behavior_std_libs__contextvars_contextvar_get_accepts_object_default() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_contextvars"
# dimension = "behavior"
# case = "contextvar_get_accepts_object_default"
# subject = "_contextvars.ContextVar.get(default)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_contextvars.pyi"
# status = "filled"
# ///
"""_contextvars.ContextVar.get(default): TypeVar defaults accept arbitrary objects."""

from _contextvars import ContextVar


class Default:
    pass


default = Default()
var = ContextVar("default_object")
assert var.get(default) is default
print("contextvar_get_accepts_object_default OK")
"###);
    assert_output(&out, r###"contextvar_get_accepts_object_default OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/_contextvars/contextvar_set_accepts_object_value.py`.
#[test]
fn test_gen_behavior_std_libs__contextvars_contextvar_set_accepts_object_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_contextvars"
# dimension = "behavior"
# case = "contextvar_set_accepts_object_value"
# subject = "_contextvars.ContextVar.set(value)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_contextvars.pyi"
# status = "filled"
# ///
"""_contextvars.ContextVar.set(value): TypeVar values accept arbitrary objects."""

from _contextvars import ContextVar, Token


class Value:
    pass


value = Value()
var = ContextVar("value_object")
token = var.set(value)
assert isinstance(token, Token)
assert var.get() is value
print("contextvar_set_accepts_object_value OK")
"###);
    assert_output(&out, r###"contextvar_set_accepts_object_value OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/_contextvars/token_has_no_exit_method.py`.
#[test]
fn test_gen_behavior_std_libs__contextvars_token_has_no_exit_method() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_contextvars"
# dimension = "behavior"
# case = "token_has_no_exit_method"
# subject = "_contextvars.Token.__exit__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_contextvars.pyi"
# status = "filled"
# ///
"""_contextvars.Token has no __exit__ method on CPython 3.12."""

from _contextvars import ContextVar

var = ContextVar("token_exit_absent")
token = var.set("value")
assert not hasattr(token, "__exit__")
try:
    token.__exit__(None, None, None)
except AttributeError:
    print("token_has_no_exit_method OK")
else:
    raise AssertionError("Token.__exit__ must be absent")
"###);
    assert_output(&out, r###"token_has_no_exit_method OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextvars/context_iteration_yields_var_value_pairs.py`.
#[test]
fn test_gen_behavior_std_libs_contextvars_context_iteration_yields_var_value_pairs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "context_iteration_yields_var_value_pairs"
# subject = "contextvars.Context"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.Context: iterating a Context yields its ContextVar keys; dict(context) maps each captured var to its value, and membership-tests by var"""
import contextvars

cv_a = contextvars.ContextVar("iter_a")
cv_b = contextvars.ContextVar("iter_b")
cv_a.set("val_a")
cv_b.set("val_b")
ctx = contextvars.copy_context()
items = dict(ctx)
assert cv_a in items, "cv_a present as a key in the context"
assert items[cv_a] == "val_a", f"cv_a value = {items[cv_a]!r}"
assert items[cv_b] == "val_b", f"cv_b value = {items[cv_b]!r}"
print("context_iteration_yields_var_value_pairs OK")
"###);
    assert_output(&out, r###"context_iteration_yields_var_value_pairs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextvars/context_run_writes_do_not_leak.py`.
#[test]
fn test_gen_behavior_std_libs_contextvars_context_run_writes_do_not_leak() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "context_run_writes_do_not_leak"
# subject = "contextvars.Context"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.Context: a set() performed inside Context.run() is visible there but does not leak back into the outer context after run() returns"""
import contextvars

cv = contextvars.ContextVar("leak", default="outer")
ctx = contextvars.copy_context()

def mutate():
    cv.set("inner")
    assert cv.get() == "inner", "mutation visible inside run"

ctx.run(mutate)
assert cv.get() == "outer", f"after run, outer unchanged = {cv.get()!r}"
print("context_run_writes_do_not_leak OK")
"###);
    assert_output(&out, r###"context_run_writes_do_not_leak OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextvars/contextvar_name_attribute.py`.
#[test]
fn test_gen_behavior_std_libs_contextvars_contextvar_name_attribute() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "contextvar_name_attribute"
# subject = "contextvars.ContextVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: a freshly constructed ContextVar exposes its name via the read-only .name attribute"""
import contextvars

cv = contextvars.ContextVar("my_var")
assert cv.name == "my_var", f"name = {cv.name!r}"
print("contextvar_name_attribute OK")
"###);
    assert_output(&out, r###"contextvar_name_attribute OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextvars/copy_context_returns_context.py`.
#[test]
fn test_gen_behavior_std_libs_contextvars_copy_context_returns_context() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "copy_context_returns_context"
# subject = "contextvars.copy_context"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.copy_context: copy_context() returns a contextvars.Context instance"""
import contextvars

ctx = contextvars.copy_context()
assert isinstance(ctx, contextvars.Context), f"copy_context() type = {type(ctx)!r}"
print("copy_context_returns_context OK")
"###);
    assert_output(&out, r###"copy_context_returns_context OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextvars/copy_context_snapshots_current_values.py`.
#[test]
fn test_gen_behavior_std_libs_contextvars_copy_context_snapshots_current_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "copy_context_snapshots_current_values"
# subject = "contextvars.Context"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.Context: a Context captured by copy_context sees the value present at copy time, not a later overwrite, when run() reads the var"""
import contextvars

cv = contextvars.ContextVar("snapshot")
cv.set("captured")
ctx = contextvars.copy_context()
cv.set("after_copy")
# Running inside the copied context sees the value at copy time, not the later one.
seen = []
ctx.run(lambda: seen.append(cv.get()))
assert seen == ["captured"], f"copy_context snapshot = {seen!r}"
print("copy_context_snapshots_current_values OK")
"###);
    assert_output(&out, r###"copy_context_snapshots_current_values OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextvars/default_returned_when_unset.py`.
#[test]
fn test_gen_behavior_std_libs_contextvars_default_returned_when_unset() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "default_returned_when_unset"
# subject = "contextvars.ContextVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: a ContextVar declared with default= returns that default from get() while no value is set"""
import contextvars

cv = contextvars.ContextVar("with_default", default=42)
assert cv.get() == 42, f"default = {cv.get()!r}"
print("default_returned_when_unset OK")
"###);
    assert_output(&out, r###"default_returned_when_unset OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextvars/multiple_vars_independent.py`.
#[test]
fn test_gen_behavior_std_libs_contextvars_multiple_vars_independent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "multiple_vars_independent"
# subject = "contextvars.ContextVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: two distinct ContextVars hold independent values; setting one does not change the other"""
import contextvars

a = contextvars.ContextVar("a", default=1)
b = contextvars.ContextVar("b", default=2)
assert a.get() != b.get(), "distinct defaults"
a.set(10)
assert a.get() == 10, "a updated"
assert b.get() == 2, "b unaffected by a.set"
print("multiple_vars_independent OK")
"###);
    assert_output(&out, r###"multiple_vars_independent OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextvars/reset_restores_previous_value.py`.
#[test]
fn test_gen_behavior_std_libs_contextvars_reset_restores_previous_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "reset_restores_previous_value"
# subject = "contextvars.ContextVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: reset(token) restores the value the ContextVar held before the matching set()"""
import contextvars

cv = contextvars.ContextVar("restore")
cv.set("first")
tok = cv.set("second")
assert cv.get() == "second", "value updated by the second set"
cv.reset(tok)
assert cv.get() == "first", "reset restores the value before the matching set"
print("reset_restores_previous_value OK")
"###);
    assert_output(&out, r###"reset_restores_previous_value OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextvars/reset_to_unset_after_first_set.py`.
#[test]
fn test_gen_behavior_std_libs_contextvars_reset_to_unset_after_first_set() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "reset_to_unset_after_first_set"
# subject = "contextvars.ContextVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: reset(token) of the first-ever set on a no-default var returns it to the unset state, so get() raises LookupError again"""
import contextvars

cv = contextvars.ContextVar("unset_after_reset")
tok = cv.set("only")
assert cv.get() == "only", "value visible while set"
cv.reset(tok)
_raised = False
try:
    cv.get()
except LookupError:
    _raised = True
assert _raised, "after resetting the first set, the var is unset again -> LookupError"
print("reset_to_unset_after_first_set OK")
"###);
    assert_output(&out, r###"reset_to_unset_after_first_set OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextvars/set_returns_token_and_updates_value.py`.
#[test]
fn test_gen_behavior_std_libs_contextvars_set_returns_token_and_updates_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "set_returns_token_and_updates_value"
# subject = "contextvars.ContextVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: set(v) returns a contextvars.Token and get() afterwards returns v"""
import contextvars

cv = contextvars.ContextVar("setget")
tok = cv.set("hello")
assert isinstance(tok, contextvars.Token), f"set() returns a Token, got {type(tok)!r}"
assert cv.get() == "hello", f"after set = {cv.get()!r}"
print("set_returns_token_and_updates_value OK")
"###);
    assert_output(&out, r###"set_returns_token_and_updates_value OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextvars/thread_starts_with_own_context.py`.
#[test]
fn test_gen_behavior_std_libs_contextvars_thread_starts_with_own_context() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "thread_starts_with_own_context"
# subject = "contextvars.ContextVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: a new thread starts with its own context: it sees the var's default (not the main thread's set value), and its own writes do not leak back to main"""
import contextvars
import threading

cv = contextvars.ContextVar("threaded", default="default")
cv.set("main_value")

saw = []

def thread_fn():
    saw.append(cv.get())  # a fresh thread starts from the default, not main's value
    cv.set("thread_val")
    saw.append(cv.get())

t = threading.Thread(target=thread_fn)
t.start()
t.join()

assert saw[0] == "default", f"thread starts at default = {saw[0]!r}"
assert saw[1] == "thread_val", f"thread sees its own write = {saw[1]!r}"
assert cv.get() == "main_value", f"main value unchanged by the thread = {cv.get()!r}"
print("thread_starts_with_own_context OK")
"###);
    assert_output(&out, r###"thread_starts_with_own_context OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextvars/token_old_value_holds_previous.py`.
#[test]
fn test_gen_behavior_std_libs_contextvars_token_old_value_holds_previous() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "token_old_value_holds_previous"
# subject = "contextvars.Token"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.Token: overwriting an already-set var yields a Token whose old_value is the prior value"""
import contextvars

cv = contextvars.ContextVar("tok_prev")
cv.set("first")
tok = cv.set("second")
assert tok.old_value == "first", f"old_value = {tok.old_value!r}"
print("token_old_value_holds_previous OK")
"###);
    assert_output(&out, r###"token_old_value_holds_previous OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextvars/token_old_value_missing_when_unset.py`.
#[test]
fn test_gen_behavior_std_libs_contextvars_token_old_value_missing_when_unset() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "token_old_value_missing_when_unset"
# subject = "contextvars.Token"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.Token: the Token from the first set() of a previously-unset var has old_value identical to contextvars.Token.MISSING"""
import contextvars

cv = contextvars.ContextVar("tok_missing")
tok = cv.set("first")
assert tok.old_value is contextvars.Token.MISSING, "old_value is MISSING when the var was previously unset"
print("token_old_value_missing_when_unset OK")
"###);
    assert_output(&out, r###"token_old_value_missing_when_unset OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextvars/token_var_points_back.py`.
#[test]
fn test_gen_behavior_std_libs_contextvars_token_var_points_back() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "token_var_points_back"
# subject = "contextvars.Token"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.Token: the Token returned by set() carries .var identical (is) to the ContextVar it came from"""
import contextvars

cv = contextvars.ContextVar("tok_var")
tok = cv.set("first")
assert tok.var is cv, "Token.var is the originating ContextVar"
print("token_var_points_back OK")
"###);
    assert_output(&out, r###"token_var_points_back OK
"###);
}
