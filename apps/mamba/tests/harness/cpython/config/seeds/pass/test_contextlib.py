# test_contextlib.py — #3446 axis-1 stdlib contextlib AssertionPass seed.
#
# Mamba-authored seed exercising the SYNC `contextlib` surface called
# out in the issue:
#   @contextmanager yield, ExitStack push/pop, suppress, redirect_stdout,
#   closing.
#
# Sync-only by design — async @asynccontextmanager is broken on mamba
# (issue #3500) and would invalidate the seed.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + public surface (hasattr).
#   2. @contextmanager generator-style CM (enter/exit ordering + yielded
#      value bound via `as`).
#   3. contextlib.suppress swallows the listed exceptions and re-raises
#      others.
#   4. contextlib.closing calls .close() on enter/exit.
#   5. contextlib.redirect_stdout captures print() output via StringIO.
#   6. ExitStack push/pop + enter_context + callback ordering (LIFO).
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: contextlib N asserts` to stdout.

import contextlib
import io

_ledger: list[int] = []

# Module-level helpers (top-level only — no nested closures).
_trace: list[str] = []


@contextlib.contextmanager
def _ctx():
    _trace.append("enter")
    yield "yielded_value"
    _trace.append("exit")


class _Closeable:
    def __init__(self):
        self.closed = False

    def close(self):
        self.closed = True


# 1. Module identity + public surface.
assert contextlib.__name__ == "contextlib", "contextlib.__name__"
_ledger.append(1)
assert hasattr(contextlib, "contextmanager"), "exposes contextmanager"
_ledger.append(1)
assert hasattr(contextlib, "ExitStack"), "exposes ExitStack"
_ledger.append(1)
assert hasattr(contextlib, "suppress"), "exposes suppress"
_ledger.append(1)
assert hasattr(contextlib, "redirect_stdout"), "exposes redirect_stdout"
_ledger.append(1)
assert hasattr(contextlib, "closing"), "exposes closing"
_ledger.append(1)
assert hasattr(contextlib, "nullcontext"), "exposes nullcontext"
_ledger.append(1)

# 2. @contextmanager generator-style CM.
with _ctx() as _val:
    assert _val == "yielded_value", "yield value bound via `as`"
    _ledger.append(1)
    assert _trace == ["enter"], "enter side-effect ran"
    _ledger.append(1)
assert _trace == ["enter", "exit"], "exit side-effect ran on with-exit"
_ledger.append(1)

# 3. contextlib.suppress — swallow listed exceptions.
_swallowed = False
with contextlib.suppress(ValueError):
    raise ValueError("expected — suppress should swallow")
    _swallowed = True  # unreachable
assert _swallowed == False, "code after raise is unreachable"
_ledger.append(1)

# Multiple exception classes — both suppressed.
with contextlib.suppress(KeyError, IndexError):
    _d = {}
    _ = _d["missing"]  # KeyError
# Re-enter; trigger the IndexError branch.
with contextlib.suppress(KeyError, IndexError):
    _l: list[int] = []
    _ = _l[5]  # IndexError

# suppress does not swallow exceptions not in the list — confirm via
# try/except wrapping the with.
_caught_unrelated = False
try:
    with contextlib.suppress(KeyError):
        raise ValueError("not in suppress list")
except ValueError:
    _caught_unrelated = True
assert _caught_unrelated == True, "suppress does not swallow unrelated exception"
_ledger.append(1)

# 4. contextlib.closing — calls .close() on exit.
_resource = _Closeable()
assert _resource.closed == False, "closing initial state"
_ledger.append(1)
with contextlib.closing(_resource) as _r:
    assert _r is _resource, "closing yields the wrapped object"
    _ledger.append(1)
    assert _r.closed == False, "closing has NOT closed mid-with-body"
    _ledger.append(1)
assert _resource.closed == True, "closing has closed after with-exit"
_ledger.append(1)

# 5. contextlib.redirect_stdout — captures print() output.
_buf = io.StringIO()
with contextlib.redirect_stdout(_buf):
    print("captured_payload")
_out = _buf.getvalue()
assert "captured_payload" in _out, "redirect_stdout captured the print output"
_ledger.append(1)
# Trailing newline from print is preserved.
assert _out.endswith("\n"), "captured output preserves the trailing newline"
_ledger.append(1)

# 6. ExitStack — enter_context + callback ordering.
_order: list[str] = []


@contextlib.contextmanager
def _track(name: str):
    _order.append(f"enter:{name}")
    yield name
    _order.append(f"exit:{name}")


with contextlib.ExitStack() as _stack:
    _a = _stack.enter_context(_track("A"))
    _b = _stack.enter_context(_track("B"))
    _stack.callback(lambda: _order.append("cb"))
    assert _a == "A", "enter_context returned the yielded value (A)"
    _ledger.append(1)
    assert _b == "B", "enter_context returned the yielded value (B)"
    _ledger.append(1)

# After ExitStack closes: callbacks fire LIFO, then context-managers exit
# in reverse-of-enter order.
assert _order[0] == "enter:A", "first enter is A"
_ledger.append(1)
assert _order[1] == "enter:B", "second enter is B"
_ledger.append(1)
assert _order[2] == "cb", "callback fires before CM exits (LIFO from top)"
_ledger.append(1)
assert _order[3] == "exit:B", "B exits first (LIFO)"
_ledger.append(1)
assert _order[4] == "exit:A", "A exits last (LIFO)"
_ledger.append(1)

# nullcontext — null op CM that yields the supplied value.
with contextlib.nullcontext("payload") as _nc:
    assert _nc == "payload", "nullcontext yields the supplied value"
    _ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: contextlib {len(_ledger)} asserts")
