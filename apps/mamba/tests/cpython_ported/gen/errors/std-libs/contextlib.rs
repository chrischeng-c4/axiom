use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/contextlib/abstract_missing_exit_not_instantiable.py`.
#[test]
fn test_gen_errors_std_libs_contextlib_abstract_missing_exit_not_instantiable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "errors"
# case = "abstract_missing_exit_not_instantiable"
# subject = "contextlib.AbstractContextManager"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.AbstractContextManager: abstract_missing_exit_not_instantiable (errors)."""
import contextlib

_raised = False
try:
    type('MissingExit', (contextlib.AbstractContextManager,), {})()
except TypeError:
    _raised = True
assert _raised, "abstract_missing_exit_not_instantiable: expected TypeError"
print("abstract_missing_exit_not_instantiable OK")
"###);
    assert_output(&out, r###"abstract_missing_exit_not_instantiable OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/contextlib/contextmanager_no_yield_raises_runtimeerror.py`.
#[test]
fn test_gen_errors_std_libs_contextlib_contextmanager_no_yield_raises_runtimeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "errors"
# case = "contextmanager_no_yield_raises_runtimeerror"
# subject = "contextlib.contextmanager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.contextmanager: a @contextmanager-decorated function that never yields raises RuntimeError when its with-block is entered (the inner generator stops immediately)"""
import contextlib


# A real generator (it contains a `yield`) that returns before ever reaching
# the yield. contextmanager.__enter__ calls next() and the generator stops
# immediately -> RuntimeError("generator didn't yield").
@contextlib.contextmanager
def no_yield():
    if False:
        yield  # makes this a generator without ever yielding
    return


_raised = False
try:
    with no_yield():
        pass
except RuntimeError as e:
    _raised = True
    assert "didn't yield" in str(e), str(e)
assert _raised, "expected RuntimeError when generator never yields"

print("contextmanager_no_yield_raises_runtimeerror OK")
"###);
    assert_output(&out, r###"contextmanager_no_yield_raises_runtimeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/contextlib/contextmanager_reraises_chained_runtimeerror.py`.
#[test]
fn test_gen_errors_std_libs_contextlib_contextmanager_reraises_chained_runtimeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "errors"
# case = "contextmanager_reraises_chained_runtimeerror"
# subject = "contextlib.contextmanager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.contextmanager: a @contextmanager that converts a body exception via `raise RuntimeError(...) from exc` propagates the new RuntimeError out, with __cause__ set to the original"""
import contextlib


@contextlib.contextmanager
def wrap():
    try:
        yield
    except Exception as exc:
        raise RuntimeError(f"caught {type(exc).__name__}") from exc


_raised = False
try:
    with wrap():
        1 / 0  # ZeroDivisionError, converted by the manager
except RuntimeError as e:
    _raised = True
    assert str(e) == "caught ZeroDivisionError", str(e)
    assert isinstance(e.__cause__, ZeroDivisionError), repr(e.__cause__)
assert _raised, "expected the converted RuntimeError to propagate"

print("contextmanager_reraises_chained_runtimeerror OK")
"###);
    assert_output(&out, r###"contextmanager_reraises_chained_runtimeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/contextlib/contextmanager_second_yield_raises_runtimeerror.py`.
#[test]
fn test_gen_errors_std_libs_contextlib_contextmanager_second_yield_raises_runtimeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "errors"
# case = "contextmanager_second_yield_raises_runtimeerror"
# subject = "contextlib.contextmanager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.contextmanager: a @contextmanager generator that yields a second time raises RuntimeError at __exit__ (the manager forbids resuming past the single yield point)"""
import contextlib


@contextlib.contextmanager
def two_yields():
    yield 1
    yield 2  # resuming the generator past the single yield is illegal


_raised = False
try:
    with two_yields():
        pass
except RuntimeError as e:
    _raised = True
    assert "didn't stop" in str(e), str(e)
assert _raised, "expected RuntimeError when generator yields a second time"

print("contextmanager_second_yield_raises_runtimeerror OK")
"###);
    assert_output(&out, r###"contextmanager_second_yield_raises_runtimeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/contextlib/contextmanager_stopiteration_passes_through.py`.
#[test]
fn test_gen_errors_std_libs_contextlib_contextmanager_stopiteration_passes_through() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "errors"
# case = "contextmanager_stopiteration_passes_through"
# subject = "contextlib.contextmanager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.contextmanager: PEP 479: a StopIteration raised inside the with-body is NOT swallowed or replaced by the manager — the same StopIteration instance propagates unchanged"""
import contextlib

stop = StopIteration("spam")


@contextlib.contextmanager
def passthrough():
    yield


_raised = False
try:
    with passthrough():
        raise stop
except StopIteration as e:
    _raised = True
    assert e is stop, "the same StopIteration instance must propagate unchanged"
assert _raised, "expected StopIteration to pass through the manager"

print("contextmanager_stopiteration_passes_through OK")
"###);
    assert_output(&out, r###"contextmanager_stopiteration_passes_through OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/contextlib/enter_context_rejects_non_cm.py`.
#[test]
fn test_gen_errors_std_libs_contextlib_enter_context_rejects_non_cm() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "errors"
# case = "enter_context_rejects_non_cm"
# subject = "contextlib.ExitStack"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.ExitStack: enter_context_rejects_non_cm (errors)."""
import contextlib

_raised = False
try:
    contextlib.ExitStack().enter_context(object())
except TypeError:
    _raised = True
assert _raised, "enter_context_rejects_non_cm: expected TypeError"
print("enter_context_rejects_non_cm OK")
"###);
    assert_output(&out, r###"enter_context_rejects_non_cm OK
"###);
}
