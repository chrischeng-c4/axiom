# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_shutil_tempfile_contextlib_weakref_inspect_value_ops"
# subject = "cpython321.test_shutil_tempfile_contextlib_weakref_inspect_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_shutil_tempfile_contextlib_weakref_inspect_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_shutil_tempfile_contextlib_weakref_inspect_value_ops: execute CPython 3.12 seed test_shutil_tempfile_contextlib_weakref_inspect_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `shutil` / `tempfile` / `contextlib` / `contextvars` /
# `weakref` / `inspect` six-pack pinned to atomic 211: `shutil`
# (the documented full module-level helper / exception
# identifier hasattr surface — `copy` / `copy2` / `copyfile` /
# `copyfileobj` / `move` / `rmtree` / `copytree` /
# `make_archive` / `disk_usage` / `which` / `get_terminal_size`
# / `Error` + the documented
# `type(shutil.disk_usage("/")).__name__ == "usage"` /
# `type(shutil.get_terminal_size()).__name__ ==
# "terminal_size"` /
# `shutil.which("python3").startswith("/")` filesystem-helper
# value contract), `tempfile` (the documented full module-
# level helper / sentinel identifier hasattr surface —
# `mkdtemp` / `mkstemp` / `NamedTemporaryFile` /
# `TemporaryFile` / `TemporaryDirectory` /
# `SpooledTemporaryFile` / `gettempdir` / `gettempprefix` /
# `tempdir` + the documented
# `type(tempfile.mkdtemp()).__name__ == "str"` /
# `len(tempfile.mkdtemp()) > 0` /
# `type(tempfile.gettempdir()).__name__ == "str"` /
# `type(tempfile.gettempprefix()).__name__ == "str"`
# temp-directory value contract), `contextlib` (the documented
# partial module-level helper identifier hasattr surface —
# `contextmanager` / `suppress` / `nullcontext` + the
# documented `contextlib.nullcontext("hello")` cm-protocol
# value contract), `contextvars` (the documented full module-level
# helper / class identifier hasattr surface — `ContextVar` /
# `Context` / `Token` / `copy_context`), `weakref` (the
# documented full module-level helper / class identifier
# hasattr surface — `ref` / `proxy` / `WeakValueDictionary` /
# `WeakKeyDictionary` / `WeakSet` / `finalize` /
# `WeakMethod`), and `inspect` (the documented partial
# module-level helper identifier hasattr surface —
# `signature` / `getmembers` / `isfunction` / `ismethod` /
# `isclass`).
#
# Behavioral edges that DIVERGE on mamba
# (hasattr(contextlib, "asynccontextmanager") / "closing" /
# "redirect_stdout" / "redirect_stderr" / "ExitStack" /
# "AsyncExitStack" / "AbstractContextManager" all False on
# mamba + contextlib.suppress(ZeroDivisionError) does not
# actually suppress the body's ZeroDivisionError on mamba,
# type(contextvars.ContextVar("x", default=10))
# .__name__ == "ContextVar" collapses to "str" on mamba +
# contextvars.ContextVar instance `.get()` method
# unavailable on mamba, hasattr(inspect, "Signature") /
# "Parameter" / "getsource" / "getsourcefile" / "getmodule"
# / "getfullargspec" / "ismodule" / "iscoroutine" /
# "isgenerator" / "isawaitable" all False on mamba +
# type(inspect.signature(fx)).__name__ == "Signature"
# collapses to "str" on mamba +
# str(inspect.signature(fx)) == "(a, b=2)" collapses to
# "()" on mamba + inspect.isfunction(fx) True collapses
# to False on mamba) are covered in the matching spec
# fixture `lang_contextlib_contextvars_inspect_silent`.
import shutil
import tempfile
import contextlib
import contextvars
import weakref
import inspect
import os


_ledger: list[int] = []

# 1) shutil — full module hasattr surface
assert hasattr(shutil, "copy") == True; _ledger.append(1)
assert hasattr(shutil, "copy2") == True; _ledger.append(1)
assert hasattr(shutil, "copyfile") == True; _ledger.append(1)
assert hasattr(shutil, "copyfileobj") == True; _ledger.append(1)
assert hasattr(shutil, "move") == True; _ledger.append(1)
assert hasattr(shutil, "rmtree") == True; _ledger.append(1)
assert hasattr(shutil, "copytree") == True; _ledger.append(1)
assert hasattr(shutil, "make_archive") == True; _ledger.append(1)
assert hasattr(shutil, "disk_usage") == True; _ledger.append(1)
assert hasattr(shutil, "which") == True; _ledger.append(1)
assert hasattr(shutil, "get_terminal_size") == True; _ledger.append(1)
assert hasattr(shutil, "Error") == True; _ledger.append(1)

# 2) shutil — filesystem-helper value contract
assert type(shutil.disk_usage("/")).__name__ == "usage"; _ledger.append(1)
assert type(shutil.get_terminal_size()).__name__ == "terminal_size"; _ledger.append(1)
assert (shutil.which("python3") or "").startswith("/"); _ledger.append(1)

# 3) tempfile — full module hasattr surface
assert hasattr(tempfile, "mkdtemp") == True; _ledger.append(1)
assert hasattr(tempfile, "mkstemp") == True; _ledger.append(1)
assert hasattr(tempfile, "NamedTemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "TemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "TemporaryDirectory") == True; _ledger.append(1)
assert hasattr(tempfile, "SpooledTemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "gettempdir") == True; _ledger.append(1)
assert hasattr(tempfile, "gettempprefix") == True; _ledger.append(1)
assert hasattr(tempfile, "tempdir") == True; _ledger.append(1)

# 4) tempfile — temp-directory value contract
_td = tempfile.mkdtemp()
assert type(_td).__name__ == "str"; _ledger.append(1)
assert len(_td) > 0; _ledger.append(1)
os.rmdir(_td)
assert type(tempfile.gettempdir()).__name__ == "str"; _ledger.append(1)
assert type(tempfile.gettempprefix()).__name__ == "str"; _ledger.append(1)

# 5) contextlib — partial module hasattr surface
#    (asynccontextmanager / closing / redirect_stdout /
#    redirect_stderr / ExitStack / AsyncExitStack /
#    AbstractContextManager DIVERGE on mamba — moved to spec)
assert hasattr(contextlib, "contextmanager") == True; _ledger.append(1)
assert hasattr(contextlib, "suppress") == True; _ledger.append(1)
assert hasattr(contextlib, "nullcontext") == True; _ledger.append(1)

# 6) contextlib — cm-protocol value contract
#    (`contextlib.suppress(ZeroDivisionError)` does not
#    actually suppress on mamba — the body's
#    `ZeroDivisionError` leaks through the __exit__ —
#    moved to spec)
with contextlib.nullcontext("hello") as _ncv:
    assert _ncv == "hello"; _ledger.append(1)

# 7) contextvars — full module hasattr surface
#    (ContextVar(default=).get() value contract + type
#    identity DIVERGE on mamba — moved to spec)
assert hasattr(contextvars, "ContextVar") == True; _ledger.append(1)
assert hasattr(contextvars, "Context") == True; _ledger.append(1)
assert hasattr(contextvars, "Token") == True; _ledger.append(1)
assert hasattr(contextvars, "copy_context") == True; _ledger.append(1)

# 8) weakref — full module hasattr surface
assert hasattr(weakref, "ref") == True; _ledger.append(1)
assert hasattr(weakref, "proxy") == True; _ledger.append(1)
assert hasattr(weakref, "WeakValueDictionary") == True; _ledger.append(1)
assert hasattr(weakref, "WeakKeyDictionary") == True; _ledger.append(1)
assert hasattr(weakref, "WeakSet") == True; _ledger.append(1)
assert hasattr(weakref, "finalize") == True; _ledger.append(1)
assert hasattr(weakref, "WeakMethod") == True; _ledger.append(1)

# 9) inspect — partial module hasattr surface
#    (Signature / Parameter / getsource / getsourcefile /
#    getmodule / getfullargspec / ismodule / iscoroutine /
#    isgenerator / isawaitable + signature value contract +
#    isfunction value contract DIVERGE on mamba — moved to
#    spec)
assert hasattr(inspect, "signature") == True; _ledger.append(1)
assert hasattr(inspect, "getmembers") == True; _ledger.append(1)
assert hasattr(inspect, "isfunction") == True; _ledger.append(1)
assert hasattr(inspect, "ismethod") == True; _ledger.append(1)
assert hasattr(inspect, "isclass") == True; _ledger.append(1)

# NB: hasattr(contextlib, "asynccontextmanager") / "closing"
# / "redirect_stdout" / "redirect_stderr" / "ExitStack" /
# "AsyncExitStack" / "AbstractContextManager" all False on
# mamba + contextlib.suppress(ZeroDivisionError) does not
# actually suppress the body's ZeroDivisionError on mamba,
# type(contextvars.ContextVar("x", default=10))
# .__name__ == "ContextVar" collapses to "str" on mamba +
# contextvars.ContextVar instance `.get()` method
# unavailable on mamba, hasattr(inspect, "Signature") /
# "Parameter" / "getsource" / "getsourcefile" / "getmodule"
# / "getfullargspec" / "ismodule" / "iscoroutine" /
# "isgenerator" / "isawaitable" all False on mamba +
# type(inspect.signature(fx)).__name__ == "Signature"
# collapses to "str" on mamba +
# str(inspect.signature(fx)) == "(a, b=2)" collapses to
# "()" on mamba + inspect.isfunction(fx) True collapses
# to False on mamba — all DIVERGE on mamba — moved to
# the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_shutil_tempfile_contextlib_weakref_inspect_value_ops {sum(_ledger)} asserts")
