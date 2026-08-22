use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/contextlib/abstract_cm_default_enter_returns_self.py`.
#[test]
fn test_gen_behavior_std_libs_contextlib_abstract_cm_default_enter_returns_self() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "abstract_cm_default_enter_returns_self"
# subject = "contextlib.AbstractContextManager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.AbstractContextManager: the default __enter__ provided by AbstractContextManager returns self, so a concrete subclass that only defines __exit__ yields itself to `as`"""
import contextlib


class Concrete(contextlib.AbstractContextManager):
    def __exit__(self, *args):
        return None


obj = Concrete()
with obj as entered:
    assert entered is obj, "default __enter__ must return self"

print("abstract_cm_default_enter_returns_self OK")
"###);
    assert_output(&out, r###"abstract_cm_default_enter_returns_self OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextlib/abstract_cm_structural_subclass.py`.
#[test]
fn test_gen_behavior_std_libs_contextlib_abstract_cm_structural_subclass() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "abstract_cm_structural_subclass"
# subject = "contextlib.AbstractContextManager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.AbstractContextManager: any class defining both __enter__ and __exit__ is a virtual subclass of AbstractContextManager via __subclasshook__; setting either to None opts the class back out"""
import contextlib


class FromScratch:
    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, tb):
        return None


# Structural subclassing: both dunders present -> virtual subclass.
assert issubclass(FromScratch, contextlib.AbstractContextManager)


# Setting __enter__ or __exit__ to None opts the class back out.
class NoEnter(FromScratch):
    __enter__ = None


class NoExit(FromScratch):
    __exit__ = None


assert not issubclass(NoEnter, contextlib.AbstractContextManager)
assert not issubclass(NoExit, contextlib.AbstractContextManager)

print("abstract_cm_structural_subclass OK")
"###);
    assert_output(&out, r###"abstract_cm_structural_subclass OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextlib/closing_calls_close_on_exception.py`.
#[test]
fn test_gen_behavior_std_libs_contextlib_closing_calls_close_on_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "closing_calls_close_on_exception"
# subject = "contextlib.closing"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.closing: contextlib.closing(obj) still calls obj.close() when the with-body raises, before re-raising the exception"""
import contextlib


class Resource:
    def __init__(self):
        self.closed = False

    def close(self):
        self.closed = True


r = Resource()
_propagated = False
try:
    with contextlib.closing(r):
        raise ZeroDivisionError("boom")
except ZeroDivisionError:
    _propagated = True
assert r.closed, "closing must call close() even when the body raises"
assert _propagated, "the body exception must still propagate"

print("closing_calls_close_on_exception OK")
"###);
    assert_output(&out, r###"closing_calls_close_on_exception OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextlib/closing_calls_close_on_exit.py`.
#[test]
fn test_gen_behavior_std_libs_contextlib_closing_calls_close_on_exit() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "closing_calls_close_on_exit"
# subject = "contextlib.closing"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.closing: contextlib.closing(obj) calls obj.close() when the with-block exits normally, and obj is not closed before the block ends"""
import contextlib


class Resource:
    def __init__(self):
        self.closed = False

    def close(self):
        self.closed = True


r = Resource()
with contextlib.closing(r) as entered:
    assert entered is r, "closing yields the wrapped object"
    assert not r.closed, "must not be closed inside the block"
assert r.closed, "closing must call close() on normal exit"

print("closing_calls_close_on_exit OK")
"###);
    assert_output(&out, r###"closing_calls_close_on_exit OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextlib/context_decorator_as_with_block.py`.
#[test]
fn test_gen_behavior_std_libs_contextlib_context_decorator_as_with_block() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "context_decorator_as_with_block"
# subject = "contextlib.ContextDecorator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.ContextDecorator: a ContextDecorator instance works as a plain with-block, running enter/body/exit in order"""
import contextlib

log: list = []


class Track(contextlib.ContextDecorator):
    def __init__(self, name):
        self.name = name

    def __enter__(self):
        log.append(f"enter:{self.name}")
        return self

    def __exit__(self, *exc):
        log.append(f"exit:{self.name}")
        return False


with Track("blk"):
    log.append("body")
assert log == ["enter:blk", "body", "exit:blk"], log

print("context_decorator_as_with_block OK")
"###);
    assert_output(&out, r###"context_decorator_as_with_block OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextlib/contextmanager_can_suppress_by_catching_yield.py`.
#[test]
fn test_gen_behavior_std_libs_contextlib_contextmanager_can_suppress_by_catching_yield() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "contextmanager_can_suppress_by_catching_yield"
# subject = "contextlib.contextmanager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.contextmanager: a @contextmanager that wraps `yield` in try/except ValueError swallows a ValueError raised in the body, so it does not propagate"""
import contextlib


@contextlib.contextmanager
def suppress_value_error():
    try:
        yield
    except ValueError:
        pass  # swallow the body's ValueError


reached_after = False
with suppress_value_error():
    raise ValueError("suppressed")  # caught by the manager, does not propagate
reached_after = True
assert reached_after, "execution must continue after a swallowed exception"

print("contextmanager_can_suppress_by_catching_yield OK")
"###);
    assert_output(&out, r###"contextmanager_can_suppress_by_catching_yield OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextlib/contextmanager_finally_runs_on_exception.py`.
#[test]
fn test_gen_behavior_std_libs_contextlib_contextmanager_finally_runs_on_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "contextmanager_finally_runs_on_exception"
# subject = "contextlib.contextmanager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.contextmanager: a @contextmanager whose body raises still runs its finally-clause cleanup before the exception propagates out of the with-block"""
import contextlib

cleaned = False


@contextlib.contextmanager
def cm():
    global cleaned
    try:
        yield
    finally:
        cleaned = True


_propagated = False
try:
    with cm():
        raise RuntimeError("boom")
except RuntimeError:
    _propagated = True
assert cleaned, "finally cleanup must run when the body raises"
assert _propagated, "the original exception must still propagate"

print("contextmanager_finally_runs_on_exception OK")
"###);
    assert_output(&out, r###"contextmanager_finally_runs_on_exception OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextlib/contextmanager_forwards_keyword_arguments.py`.
#[test]
fn test_gen_behavior_std_libs_contextlib_contextmanager_forwards_keyword_arguments() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "contextmanager_forwards_keyword_arguments"
# subject = "contextlib.contextmanager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.contextmanager: a @contextmanager forwards positional and keyword arguments verbatim to the wrapped generator (even argument names like self/func/args/kwds)"""
import contextlib


@contextlib.contextmanager
def forward(self, func, args, kwds):
    yield (self, func, args, kwds)


with forward(self=11, func=22, args=33, kwds=44) as target:
    assert target == (11, 22, 33, 44), f"kwarg forward = {target!r}"

print("contextmanager_forwards_keyword_arguments OK")
"###);
    assert_output(&out, r###"contextmanager_forwards_keyword_arguments OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextlib/contextmanager_result_is_decorator.py`.
#[test]
fn test_gen_behavior_std_libs_contextlib_contextmanager_result_is_decorator() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "contextmanager_result_is_decorator"
# subject = "contextlib.contextmanager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.contextmanager: a @contextmanager result is itself a ContextDecorator in 3.12, so it can decorate a function and wrap each call in enter/exit"""
import contextlib

log: list = []


@contextlib.contextmanager
def managed():
    log.append("cm_enter")
    yield
    log.append("cm_exit")


@managed()
def cm_decorated():
    log.append("cm_body")


cm_decorated()
assert log == ["cm_enter", "cm_body", "cm_exit"], log

print("contextmanager_result_is_decorator OK")
"###);
    assert_output(&out, r###"contextmanager_result_is_decorator OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextlib/contextmanager_yields_value_in_order.py`.
#[test]
fn test_gen_behavior_std_libs_contextlib_contextmanager_yields_value_in_order() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "contextmanager_yields_value_in_order"
# subject = "contextlib.contextmanager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.contextmanager: a @contextmanager runs the pre-yield body on enter, yields its value to `as`, then runs the post-yield body on exit, in that order (before/during/after)"""
import contextlib

order: list = []


@contextlib.contextmanager
def cm():
    order.append("before")
    yield "value"
    order.append("after")


with cm() as v:
    assert v == "value", f"yield value = {v!r}"
    order.append("during")
assert order == ["before", "during", "after"], f"order = {order!r}"

print("contextmanager_yields_value_in_order OK")
"###);
    assert_output(&out, r###"contextmanager_yields_value_in_order OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextlib/exitstack_callbacks_run_lifo.py`.
#[test]
fn test_gen_behavior_std_libs_contextlib_exitstack_callbacks_run_lifo() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "exitstack_callbacks_run_lifo"
# subject = "contextlib.ExitStack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.ExitStack: ExitStack.callback returns the registered function unchanged and replays stored args/kwargs at unwind time, in LIFO order"""
import contextlib

calls: list = []


def record(*args, **kwds):
    calls.append((args, kwds))


with contextlib.ExitStack() as stack:
    returned = stack.callback(record, 1, key="v")
    stack.callback(record, 2)
    assert returned is record, "callback returns the function unchanged"
# LIFO: the second-registered callback fires first.
assert calls == [((2,), {}), ((1,), {"key": "v"})], calls

print("exitstack_callbacks_run_lifo OK")
"###);
    assert_output(&out, r###"exitstack_callbacks_run_lifo OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextlib/exitstack_enter_context_exits_lifo.py`.
#[test]
fn test_gen_behavior_std_libs_contextlib_exitstack_enter_context_exits_lifo() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "exitstack_enter_context_exits_lifo"
# subject = "contextlib.ExitStack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.ExitStack: ExitStack.enter_context enters each context manager and exits them in LIFO order when the stack closes"""
import contextlib

exits: list = []


@contextlib.contextmanager
def track_exit(name: str):
    try:
        yield
    finally:
        exits.append(name)


with contextlib.ExitStack() as stack:
    stack.enter_context(track_exit("a"))
    stack.enter_context(track_exit("b"))
    stack.enter_context(track_exit("c"))
# LIFO: last entered exits first.
assert exits == ["c", "b", "a"], f"ExitStack LIFO = {exits!r}"

print("exitstack_enter_context_exits_lifo OK")
"###);
    assert_output(&out, r###"exitstack_enter_context_exits_lifo OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextlib/exitstack_pop_all_transfers_callbacks.py`.
#[test]
fn test_gen_behavior_std_libs_contextlib_exitstack_pop_all_transfers_callbacks() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "exitstack_pop_all_transfers_callbacks"
# subject = "contextlib.ExitStack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.ExitStack: ExitStack.pop_all moves the registered callbacks to a fresh stack; closing the original is then a no-op and closing the new stack fires the callbacks"""
import contextlib

fired: list = []

es = contextlib.ExitStack()
es.callback(fired.append, "cb")
new_es = es.pop_all()

# After pop_all, the original holds nothing: closing it is a no-op.
es.close()
assert fired == [], "original stack must be empty after pop_all"

# The transferred callback fires only when the new stack closes.
new_es.close()
assert fired == ["cb"], fired

print("exitstack_pop_all_transfers_callbacks OK")
"###);
    assert_output(&out, r###"exitstack_pop_all_transfers_callbacks OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextlib/exitstack_propagation_order_keeps_earliest.py`.
#[test]
fn test_gen_behavior_std_libs_contextlib_exitstack_propagation_order_keeps_earliest() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "exitstack_propagation_order_keeps_earliest"
# subject = "contextlib.ExitStack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.ExitStack: with callbacks running LIFO, a suppressor swallows the later-raised exception and the earlier raised exception is the one that propagates out"""
import contextlib


def raise_exc(exc):
    raise exc


def suppress_all(*exc_details):
    return True


# Callbacks unwind LIFO: the last-registered IndexError raiser runs first, the
# suppressor swallows it, and the first-registered KeyError raiser runs last —
# so KeyError is what propagates.
caught = None
try:
    with contextlib.ExitStack() as stack:
        stack.callback(raise_exc, KeyError("earliest"))
        stack.push(suppress_all)
        stack.callback(raise_exc, IndexError("latest"))
except Exception as exc:
    caught = exc
assert isinstance(caught, KeyError), type(caught).__name__

print("exitstack_propagation_order_keeps_earliest OK")
"###);
    assert_output(&out, r###"exitstack_propagation_order_keeps_earliest OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextlib/exitstack_push_registers_raw_exit_callable.py`.
#[test]
fn test_gen_behavior_std_libs_contextlib_exitstack_push_registers_raw_exit_callable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "exitstack_push_registers_raw_exit_callable"
# subject = "contextlib.ExitStack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.ExitStack: ExitStack.push registers a raw __exit__-style callable (bypassing the __enter__ check) and a push()ed callback returning True suppresses a body exception"""
import contextlib

seen: list = []


def exit_cb(*exc_details):
    seen.append(exc_details[0])  # the exc_type, or None on clean exit
    return False


# push() takes a raw __exit__-style callable; on a clean exit it is called
# with (None, None, None).
with contextlib.ExitStack() as stack:
    stack.push(exit_cb)
assert seen == [None], seen


# A push()ed callback returning True suppresses a body exception.
def suppress_all(*exc_details):
    return True


with contextlib.ExitStack() as stack:
    stack.push(suppress_all)
    1 / 0  # suppressed by the callback

print("exitstack_push_registers_raw_exit_callable OK")
"###);
    assert_output(&out, r###"exitstack_push_registers_raw_exit_callable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextlib/nullcontext_yields_its_argument.py`.
#[test]
fn test_gen_behavior_std_libs_contextlib_nullcontext_yields_its_argument() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "nullcontext_yields_its_argument"
# subject = "contextlib.nullcontext"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.nullcontext: nullcontext(value) yields the value to `as` and is a no-op manager; nullcontext() with no argument yields None"""
import contextlib

with contextlib.nullcontext("token") as tok:
    assert tok == "token", f"nullcontext value = {tok!r}"

with contextlib.nullcontext(42) as n:
    assert n == 42, f"nullcontext value = {n!r}"

with contextlib.nullcontext() as none:
    assert none is None, f"nullcontext() with no arg = {none!r}"

print("nullcontext_yields_its_argument OK")
"###);
    assert_output(&out, r###"nullcontext_yields_its_argument OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextlib/redirect_stderr_captures_stderr.py`.
#[test]
fn test_gen_behavior_std_libs_contextlib_redirect_stderr_captures_stderr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "redirect_stderr_captures_stderr"
# subject = "contextlib.redirect_stderr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.redirect_stderr: redirect_stderr(buf) routes writes to sys.stderr into buf for the duration of the with-block"""
import contextlib
import io
import sys

buf = io.StringIO()
with contextlib.redirect_stderr(buf):
    print("error output", file=sys.stderr)
assert "error output" in buf.getvalue(), f"redirect_stderr = {buf.getvalue()!r}"

print("redirect_stderr_captures_stderr OK")
"###);
    assert_output(&out, r###"redirect_stderr_captures_stderr OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextlib/redirect_stdout_captures_print.py`.
#[test]
fn test_gen_behavior_std_libs_contextlib_redirect_stdout_captures_print() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "redirect_stdout_captures_print"
# subject = "contextlib.redirect_stdout"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.redirect_stdout: redirect_stdout(buf) routes print() output into buf for the duration of the with-block and yields buf as the enter result"""
import contextlib
import io

buf = io.StringIO()
with contextlib.redirect_stdout(buf) as entered:
    assert entered is buf, "redirect_stdout yields the target stream"
    print("captured")
# Outside the block stdout is restored.
assert buf.getvalue() == "captured\n", f"redirect = {buf.getvalue()!r}"

print("redirect_stdout_captures_print OK")
"###);
    assert_output(&out, r###"redirect_stdout_captures_print OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextlib/redirect_stdout_reusable_restores_stream.py`.
#[test]
fn test_gen_behavior_std_libs_contextlib_redirect_stdout_reusable_restores_stream() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "redirect_stdout_reusable_restores_stream"
# subject = "contextlib.redirect_stdout"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.redirect_stdout: a single redirect_stdout(buf) instance is reusable: re-entering keeps writing to the same target and sys.stdout is restored after each block"""
import contextlib
import io
import sys

buf = io.StringIO()
redir = contextlib.redirect_stdout(buf)
saved = sys.stdout
with redir:
    print("Hello", end=" ")
with redir:
    print("World!")
assert sys.stdout is saved, "stdout must be restored after reuse"
assert buf.getvalue() == "Hello World!\n", repr(buf.getvalue())

print("redirect_stdout_reusable_restores_stream OK")
"###);
    assert_output(&out, r###"redirect_stdout_reusable_restores_stream OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextlib/suppress_catches_listed_exceptions_only.py`.
#[test]
fn test_gen_behavior_std_libs_contextlib_suppress_catches_listed_exceptions_only() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "suppress_catches_listed_exceptions_only"
# subject = "contextlib.suppress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.suppress: suppress(KeyError, IndexError) swallows a listed exception but lets an unlisted ValueError propagate out of the with-block"""
import contextlib

# A listed exception is swallowed.
with contextlib.suppress(KeyError, IndexError):
    raise KeyError("suppressed")

# An unlisted exception propagates.
_propagated = False
try:
    with contextlib.suppress(KeyError):
        raise ValueError("not suppressed")
except ValueError:
    _propagated = True
assert _propagated, "an unlisted exception must propagate"

print("suppress_catches_listed_exceptions_only OK")
"###);
    assert_output(&out, r###"suppress_catches_listed_exceptions_only OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/contextlib/suppress_is_reusable_and_reentrant.py`.
#[test]
fn test_gen_behavior_std_libs_contextlib_suppress_is_reusable_and_reentrant() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "suppress_is_reusable_and_reentrant"
# subject = "contextlib.suppress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.suppress: a single suppress() instance can be entered repeatedly (reusable) and nested within itself (reentrant); the outer block resumes after an inner block swallows an exception"""
import contextlib

ignore = contextlib.suppress(Exception)

# REUSABLE: the same instance entered more than once.
with ignore:
    pass
with ignore:
    len(5)  # TypeError, suppressed

# REENTRANT: nesting the same instance; the outer block resumes after the
# inner one swallows an exception.
outer_continued = False
with ignore:
    with ignore:
        len(5)  # suppressed by inner
    outer_continued = True
    1 / 0  # suppressed by outer
assert outer_continued, "outer block must resume after inner suppress"

print("suppress_is_reusable_and_reentrant OK")
"###);
    assert_output(&out, r###"suppress_is_reusable_and_reentrant OK
"###);
}
