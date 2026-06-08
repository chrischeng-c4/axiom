# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_contextvars_contextlib_weakref_gc_atexit_value_ops"
# subject = "cpython321.test_contextvars_contextlib_weakref_gc_atexit_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_contextvars_contextlib_weakref_gc_atexit_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_contextvars_contextlib_weakref_gc_atexit_value_ops: execute CPython 3.12 seed test_contextvars_contextlib_weakref_gc_atexit_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `contextvars` / `contextlib` / `weakref` / `gc` / `atexit`
# five-pack pinned to atomic 200: `contextvars` (the documented
# full module-level class / function identifier hasattr surface —
# `ContextVar` / `Context` / `copy_context` / `Token`),
# `contextlib` (the documented partial module-level decorator /
# class identifier hasattr surface — `contextmanager` /
# `nullcontext` / `suppress`), `weakref` (the documented full
# module-level class / function identifier hasattr surface —
# `ref` / `proxy` / `WeakKeyDictionary` / `WeakValueDictionary`
# / `WeakSet` / `WeakMethod` / `finalize` / `getweakrefcount` /
# `getweakrefs` / `ProxyType` / `ProxyTypes` / `ReferenceType` /
# `CallableProxyType` + the documented
# `type(weakref.ref(obj)).__name__ == "ReferenceType"`
# instance class identity contract), `gc` (the documented
# partial module-level helper / debug-constant identifier
# hasattr surface — `collect` / `enable` / `disable` /
# `isenabled` / `get_count` / `get_threshold` / `set_threshold`
# / `get_objects` / `get_stats` / `freeze` / `unfreeze` /
# `DEBUG_LEAK` / `DEBUG_STATS` / `DEBUG_COLLECTABLE` /
# `DEBUG_UNCOLLECTABLE` / `DEBUG_SAVEALL` + the documented
# `type(gc.isenabled()) is bool` / `type(gc.collect()) is int` /
# `type(gc.get_count()) is tuple` / `len(gc.get_count()) == 3`
# return-type contracts), and `atexit` (the documented full
# module-level register / unregister identifier hasattr
# surface — `register` / `unregister` / `_run_exitfuncs` /
# `_clear`).
#
# Behavioral edges that DIVERGE on mamba
# (type(contextvars.ContextVar("x", default=42)).__name__
# collapses to "str" on mamba instead of "ContextVar",
# contextvars.ContextVar("x", default=42).name returns None on
# mamba instead of "x", hasattr(contextlib, "asynccontextmanager")
# / "closing" / "redirect_stdout" / "redirect_stderr" /
# "ExitStack" / "AsyncExitStack" / "AbstractContextManager" /
# "AbstractAsyncContextManager" / "ContextDecorator" all False
# on mamba, weakref.ref(obj)() is obj returns False on mamba,
# weakref.getweakrefcount(obj) returns 0 on mamba, hasattr(gc,
# "get_referrers") / "get_referents" / "garbage" / "callbacks"
# all False on mamba) are covered in the matching spec fixture
# `lang_contextvars_weakref_gc_silent`.
import contextvars
import contextlib
import weakref
import gc
import atexit


class _WeakHolder:
    pass


_ledger: list[int] = []

# 1) contextvars — full module hasattr surface
assert hasattr(contextvars, "ContextVar") == True; _ledger.append(1)
assert hasattr(contextvars, "Context") == True; _ledger.append(1)
assert hasattr(contextvars, "copy_context") == True; _ledger.append(1)
assert hasattr(contextvars, "Token") == True; _ledger.append(1)

# 2) contextlib — partial module hasattr surface
#    (asynccontextmanager / closing / redirect_stdout /
#    redirect_stderr / ExitStack / AsyncExitStack /
#    AbstractContextManager / AbstractAsyncContextManager /
#    ContextDecorator all DIVERGE on mamba — moved to spec)
assert hasattr(contextlib, "contextmanager") == True; _ledger.append(1)
assert hasattr(contextlib, "nullcontext") == True; _ledger.append(1)
assert hasattr(contextlib, "suppress") == True; _ledger.append(1)

# 3) weakref — full module hasattr surface
assert hasattr(weakref, "ref") == True; _ledger.append(1)
assert hasattr(weakref, "proxy") == True; _ledger.append(1)
assert hasattr(weakref, "WeakKeyDictionary") == True; _ledger.append(1)
assert hasattr(weakref, "WeakValueDictionary") == True; _ledger.append(1)
assert hasattr(weakref, "WeakSet") == True; _ledger.append(1)
assert hasattr(weakref, "WeakMethod") == True; _ledger.append(1)
assert hasattr(weakref, "finalize") == True; _ledger.append(1)
assert hasattr(weakref, "getweakrefcount") == True; _ledger.append(1)
assert hasattr(weakref, "getweakrefs") == True; _ledger.append(1)
assert hasattr(weakref, "ProxyType") == True; _ledger.append(1)
assert hasattr(weakref, "ProxyTypes") == True; _ledger.append(1)
assert hasattr(weakref, "ReferenceType") == True; _ledger.append(1)
assert hasattr(weakref, "CallableProxyType") == True; _ledger.append(1)

# 4) weakref.ref — instance class identity contract
_obj = _WeakHolder()
_r = weakref.ref(_obj)
assert type(_r).__name__ == "ReferenceType"; _ledger.append(1)

# 5) gc — partial module hasattr surface
#    (get_referrers / get_referents / garbage / callbacks
#    all DIVERGE on mamba — moved to spec)
assert hasattr(gc, "collect") == True; _ledger.append(1)
assert hasattr(gc, "enable") == True; _ledger.append(1)
assert hasattr(gc, "disable") == True; _ledger.append(1)
assert hasattr(gc, "isenabled") == True; _ledger.append(1)
assert hasattr(gc, "get_count") == True; _ledger.append(1)
assert hasattr(gc, "get_threshold") == True; _ledger.append(1)
assert hasattr(gc, "set_threshold") == True; _ledger.append(1)
assert hasattr(gc, "get_objects") == True; _ledger.append(1)
assert hasattr(gc, "get_stats") == True; _ledger.append(1)
assert hasattr(gc, "freeze") == True; _ledger.append(1)
assert hasattr(gc, "unfreeze") == True; _ledger.append(1)
assert hasattr(gc, "DEBUG_LEAK") == True; _ledger.append(1)
assert hasattr(gc, "DEBUG_STATS") == True; _ledger.append(1)
assert hasattr(gc, "DEBUG_COLLECTABLE") == True; _ledger.append(1)
assert hasattr(gc, "DEBUG_UNCOLLECTABLE") == True; _ledger.append(1)
assert hasattr(gc, "DEBUG_SAVEALL") == True; _ledger.append(1)

# 6) gc — return-type contract
assert type(gc.isenabled()).__name__ == "bool"; _ledger.append(1)
assert type(gc.collect()).__name__ == "int"; _ledger.append(1)
assert type(gc.get_count()).__name__ == "tuple"; _ledger.append(1)
assert len(gc.get_count()) == 3; _ledger.append(1)

# 7) atexit — full module hasattr surface
assert hasattr(atexit, "register") == True; _ledger.append(1)
assert hasattr(atexit, "unregister") == True; _ledger.append(1)
assert hasattr(atexit, "_run_exitfuncs") == True; _ledger.append(1)
assert hasattr(atexit, "_clear") == True; _ledger.append(1)

# NB: type(contextvars.ContextVar("x", default=42)).__name__
# collapses to "str" on mamba instead of "ContextVar",
# contextvars.ContextVar("x", default=42).name returns None
# on mamba instead of "x", hasattr(contextlib,
# "asynccontextmanager") / "closing" / "redirect_stdout" /
# "redirect_stderr" / "ExitStack" / "AsyncExitStack" /
# "AbstractContextManager" / "AbstractAsyncContextManager" /
# "ContextDecorator" all False on mamba, weakref.ref(obj)() is
# obj returns False on mamba, weakref.getweakrefcount(obj)
# returns 0 on mamba, hasattr(gc, "get_referrers") /
# "get_referents" / "garbage" / "callbacks" all False on
# mamba — all DIVERGE on mamba — moved to the divergence-spec
# fixture.

print(f"MAMBA_ASSERTION_PASS: test_contextvars_contextlib_weakref_gc_atexit_value_ops {sum(_ledger)} asserts")
