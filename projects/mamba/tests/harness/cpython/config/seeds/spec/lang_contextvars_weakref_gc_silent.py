# Operational AssertionPass seed for SILENT divergences across
# the `contextvars` instance class identity / instance `.name`
# round-trip contract + `contextlib` module identifier surface
# + `weakref` referent-recovery / live-count contract + `gc`
# module identifier surface pinned by atomic 200:
# `contextvars.ContextVar` (the documented class identity —
# `type(contextvars.ContextVar("x", default=42)).__name__
# == "ContextVar"` on CPython; mamba collapses to "str" via
# the integer-handle pattern — and the documented
# instance `.name` attribute round-trip — mamba returns
# None), `contextlib` (the documented decorator / class
# identifier surface — `asynccontextmanager` / `closing` /
# `redirect_stdout` / `redirect_stderr` / `ExitStack` /
# `AsyncExitStack` / `AbstractContextManager` /
# `AbstractAsyncContextManager` / `ContextDecorator`),
# `weakref` (the documented `weakref.ref(obj)() is obj`
# referent-recovery contract — mamba returns False — and the
# documented `weakref.getweakrefcount(obj) == 1` live-count
# contract — mamba returns 0), and `gc` (the documented helper
# / sentinel identifier surface — `get_referrers` /
# `get_referents` / `garbage` / `callbacks`).
#
# The matching subset (full contextvars hasattr,
# partial contextlib hasattr, full weakref hasattr +
# ReferenceType identity, partial gc hasattr +
# isenabled / collect / get_count return-type contract,
# full atexit hasattr) is covered by
# `test_contextvars_contextlib_weakref_gc_atexit_value_ops`;
# this fixture pins the CPython-only contracts that mamba
# currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • type(contextvars.ContextVar("x", default=42)).__name__
#     == "ContextVar" — documented class identity
#     (mamba: "str" via integer-handle pattern);
#   • contextvars.ContextVar("x", default=42).name == "x" —
#     documented instance attribute round-trip
#     (mamba: None);
#   • hasattr(contextlib, "asynccontextmanager") is True —
#     documented decorator identifier (mamba: False);
#   • hasattr(contextlib, "closing") is True — documented
#     class identifier (mamba: False);
#   • hasattr(contextlib, "redirect_stdout") is True —
#     documented class identifier (mamba: False);
#   • hasattr(contextlib, "redirect_stderr") is True —
#     documented class identifier (mamba: False);
#   • hasattr(contextlib, "ExitStack") is True — documented
#     class identifier (mamba: False);
#   • hasattr(contextlib, "AsyncExitStack") is True —
#     documented class identifier (mamba: False);
#   • hasattr(contextlib, "AbstractContextManager") is True —
#     documented class identifier (mamba: False);
#   • hasattr(contextlib, "AbstractAsyncContextManager") is
#     True — documented class identifier (mamba: False);
#   • hasattr(contextlib, "ContextDecorator") is True —
#     documented class identifier (mamba: False);
#   • weakref.ref(obj)() is obj — documented referent
#     recovery (mamba: False);
#   • weakref.getweakrefcount(obj) == 1 — documented live
#     reference count (mamba: 0);
#   • hasattr(gc, "get_referrers") is True — documented
#     helper identifier (mamba: False);
#   • hasattr(gc, "get_referents") is True — documented
#     helper identifier (mamba: False);
#   • hasattr(gc, "garbage") is True — documented sentinel
#     identifier (mamba: False);
#   • hasattr(gc, "callbacks") is True — documented sentinel
#     identifier (mamba: False).
import contextvars as _contextvars_mod
import contextlib as _contextlib_mod
import weakref as _weakref_mod
import gc as _gc_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identity / module-attribute identifier behavior that
# mamba's bundled type stubs do not surface accurately.
contextvars: Any = _contextvars_mod
contextlib: Any = _contextlib_mod
weakref: Any = _weakref_mod
gc: Any = _gc_mod


class _WeakHolder:
    pass


_ledger: list[int] = []

# 1) contextvars.ContextVar — instance class identity contract
_cv = contextvars.ContextVar("x", default=42)
assert type(_cv).__name__ == "ContextVar"; _ledger.append(1)

# 2) contextvars.ContextVar — instance .name round-trip
assert _cv.name == "x"; _ledger.append(1)

# 3) contextlib — module identifier surface
assert hasattr(contextlib, "asynccontextmanager") == True; _ledger.append(1)
assert hasattr(contextlib, "closing") == True; _ledger.append(1)
assert hasattr(contextlib, "redirect_stdout") == True; _ledger.append(1)
assert hasattr(contextlib, "redirect_stderr") == True; _ledger.append(1)
assert hasattr(contextlib, "ExitStack") == True; _ledger.append(1)
assert hasattr(contextlib, "AsyncExitStack") == True; _ledger.append(1)
assert hasattr(contextlib, "AbstractContextManager") == True; _ledger.append(1)
assert hasattr(contextlib, "AbstractAsyncContextManager") == True; _ledger.append(1)
assert hasattr(contextlib, "ContextDecorator") == True; _ledger.append(1)

# 4) weakref — referent recovery + live count contract
_obj = _WeakHolder()
_r = weakref.ref(_obj)
assert (_r() is _obj) == True; _ledger.append(1)
assert weakref.getweakrefcount(_obj) == 1; _ledger.append(1)

# 5) gc — module identifier surface
assert hasattr(gc, "get_referrers") == True; _ledger.append(1)
assert hasattr(gc, "get_referents") == True; _ledger.append(1)
assert hasattr(gc, "garbage") == True; _ledger.append(1)
assert hasattr(gc, "callbacks") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_contextvars_weakref_gc_silent {sum(_ledger)} asserts")
