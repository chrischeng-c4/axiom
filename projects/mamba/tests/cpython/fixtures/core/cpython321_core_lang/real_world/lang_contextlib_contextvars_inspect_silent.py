# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_contextlib_contextvars_inspect_silent"
# subject = "cpython321.lang_contextlib_contextvars_inspect_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_contextlib_contextvars_inspect_silent.py"
# status = "filled"
# ///
"""cpython321.lang_contextlib_contextvars_inspect_silent: execute CPython 3.12 seed lang_contextlib_contextvars_inspect_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the `contextlib` /
# `contextvars` / `inspect` three-pack pinned to atomic 211:
# `contextlib` (the documented
# `hasattr(contextlib, "asynccontextmanager") / "closing" /
# "redirect_stdout" / "redirect_stderr" / "ExitStack" /
# "AsyncExitStack" / "AbstractContextManager" == True`
# extended hasattr surface + the documented
# `with contextlib.suppress(ZeroDivisionError): 1 / 0`
# silently-catching cm-protocol value contract),
# `contextvars` (the documented
# `type(contextvars.ContextVar("x", default=10)).__name__
# == "ContextVar"` constructor-identity value contract +
# the documented `contextvars.ContextVar("x", default=10)
# .get() == 10` instance-method value contract), and
# `inspect` (the documented
# `hasattr(inspect, "Signature") / "Parameter" /
# "getsource" / "getsourcefile" / "getmodule" /
# "getfullargspec" / "ismodule" / "iscoroutine" /
# "isgenerator" / "isawaitable" == True` extended hasattr
# surface + the documented
# `type(inspect.signature(fx)).__name__ == "Signature"` /
# `str(inspect.signature(fx)) == "(a, b=2)"`
# signature-class-identity value contract + the documented
# `inspect.isfunction(fx) == True` function-predicate
# value contract).
#
# Behavioral edges that CONFORM on mamba
# (contextlib `contextmanager` / `suppress` / `nullcontext`
# hasattr surface + `contextlib.nullcontext("hello")`
# cm-protocol value contract, contextvars `ContextVar` /
# `Context` / `Token` / `copy_context` hasattr surface,
# inspect `signature` / `getmembers` / `isfunction` /
# `ismethod` / `isclass` hasattr surface) are covered in
# the matching pass fixture
# `test_shutil_tempfile_contextlib_weakref_inspect_value_ops`.
from typing import Any
import contextlib as _contextlib_mod
import contextvars as _contextvars_mod
import inspect as _inspect_mod

contextlib: Any = _contextlib_mod
contextvars: Any = _contextvars_mod
inspect: Any = _inspect_mod


_ledger: list[int] = []

# 1) contextlib — extended module hasattr surface
#    (mamba: asynccontextmanager / closing / redirect_stdout
#    / redirect_stderr / ExitStack / AsyncExitStack /
#    AbstractContextManager all False)
assert hasattr(contextlib, "asynccontextmanager") == True; _ledger.append(1)
assert hasattr(contextlib, "closing") == True; _ledger.append(1)
assert hasattr(contextlib, "redirect_stdout") == True; _ledger.append(1)
assert hasattr(contextlib, "redirect_stderr") == True; _ledger.append(1)
assert hasattr(contextlib, "ExitStack") == True; _ledger.append(1)
assert hasattr(contextlib, "AsyncExitStack") == True; _ledger.append(1)
assert hasattr(contextlib, "AbstractContextManager") == True; _ledger.append(1)

# 2) contextlib — silently-catching cm-protocol value contract
#    (mamba: contextlib.suppress(ZeroDivisionError) does not
#    actually catch the body's ZeroDivisionError — the
#    division-by-zero propagates out of the with-block
#    instead of being swallowed)
_supp_caught = False
try:
    with contextlib.suppress(ZeroDivisionError):
        _ = 1 / 0
    _supp_caught = True
except ZeroDivisionError:
    _supp_caught = False
assert _supp_caught == True; _ledger.append(1)

# 3) contextvars — ContextVar constructor-identity value contract
#    (mamba: ContextVar(...) returns a `str` instead of a
#    `ContextVar` instance, and the resulting object has no
#    `.get()` method)
_cv = contextvars.ContextVar("x", default=10)
assert type(_cv).__name__ == "ContextVar"; _ledger.append(1)
assert _cv.get() == 10; _ledger.append(1)

# 4) inspect — extended module hasattr surface
#    (mamba: Signature / Parameter / getsource / getsourcefile
#    / getmodule / getfullargspec / ismodule / iscoroutine /
#    isgenerator / isawaitable all False)
assert hasattr(inspect, "Signature") == True; _ledger.append(1)
assert hasattr(inspect, "Parameter") == True; _ledger.append(1)
assert hasattr(inspect, "getsource") == True; _ledger.append(1)
assert hasattr(inspect, "getsourcefile") == True; _ledger.append(1)
assert hasattr(inspect, "getmodule") == True; _ledger.append(1)
assert hasattr(inspect, "getfullargspec") == True; _ledger.append(1)
assert hasattr(inspect, "ismodule") == True; _ledger.append(1)
assert hasattr(inspect, "iscoroutine") == True; _ledger.append(1)
assert hasattr(inspect, "isgenerator") == True; _ledger.append(1)
assert hasattr(inspect, "isawaitable") == True; _ledger.append(1)


def _fx_for_signature(a, b=2):
    pass


# 5) inspect — signature-class-identity value contract
#    (mamba: type(inspect.signature(fx)).__name__ collapses
#    to "str" + str(inspect.signature(fx)) == "(a, b=2)"
#    collapses to "()")
_sig = inspect.signature(_fx_for_signature)
assert type(_sig).__name__ == "Signature"; _ledger.append(1)
assert str(_sig) == "(a, b=2)"; _ledger.append(1)

# 6) inspect — function-predicate value contract
#    (mamba: inspect.isfunction(fx) collapses to False even
#    for a module-level def)
assert inspect.isfunction(_fx_for_signature) == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_contextlib_contextvars_inspect_silent {sum(_ledger)} asserts")
