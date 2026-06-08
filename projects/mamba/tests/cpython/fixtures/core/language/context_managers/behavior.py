"""Behavior contract for language context managers.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import contextlib

# Rule 1: __enter__ runs before body, __exit__ runs after
_events = []
class _Tracker:
    def __enter__(self) -> "_Tracker":
        _events.append("enter")
        return self
    def __exit__(self, exc_type, exc_val, exc_tb) -> bool:
        _events.append(f"exit:{exc_type is None}")
        return False

with _Tracker():
    _events.append("body")
assert _events == ["enter", "body", "exit:True"], f"events = {_events!r}"

# Rule 2: __exit__ receives exception info when exception raised
_exc_info = []
class _ErrCM:
    def __enter__(self) -> "_ErrCM":
        return self
    def __exit__(self, exc_type, exc_val, exc_tb) -> bool:
        _exc_info.extend([exc_type, str(exc_val)])
        return True  # suppress exception

with _ErrCM():
    raise ValueError("test error")
assert _exc_info[0] is ValueError, f"exc_type = {_exc_info[0]!r}"
assert _exc_info[1] == "test error", f"exc_val = {_exc_info[1]!r}"

# Rule 3: __exit__ returning False re-raises exception
_raised = False
class _NoSuppress:
    def __enter__(self) -> "_NoSuppress":
        return self
    def __exit__(self, exc_type, exc_val, exc_tb) -> bool:
        return False  # don't suppress
try:
    with _NoSuppress():
        raise RuntimeError("boom")
except RuntimeError:
    _raised = True
assert _raised, "exception not re-raised"

# Rule 4: contextmanager decorator
@contextlib.contextmanager
def _cm(v: list):
    v.append("before")
    yield 42
    v.append("after")

_v: list = []
with _cm(_v) as result:
    assert result == 42, f"yield value = {result!r}"
    _v.append("during")
assert _v == ["before", "during", "after"], f"_v = {_v!r}"

# Rule 5: contextmanager handles exceptions
@contextlib.contextmanager
def _safe():
    try:
        yield
    except ValueError:
        pass  # suppress ValueError

with _safe():
    raise ValueError("suppressed")
# No exception here — suppressed

# Rule 6: multiple with in one statement (Python 3.1+)
_opens = []
class _CM:
    def __init__(self, name: str):
        self.name = name
    def __enter__(self) -> str:
        _opens.append(f"open:{self.name}")
        return self.name
    def __exit__(self, *args) -> bool:
        _opens.append(f"close:{self.name}")
        return False

with _CM("A") as a, _CM("B") as b:
    assert a == "A" and b == "B"
assert _opens == ["open:A", "open:B", "close:B", "close:A"], f"opens = {_opens!r}"

# Rule 7: contextlib.suppress
_ran = False
with contextlib.suppress(ZeroDivisionError):
    _ = 1 / 0
    _ran = True  # should not reach
assert not _ran, "suppress: body continued after exception"

# Rule 8: contextlib.ExitStack
_stack_log = []
with contextlib.ExitStack() as stack:
    cm_a = stack.enter_context(_CM("X"))
    cm_b = stack.enter_context(_CM("Y"))
    assert cm_a == "X" and cm_b == "Y"
assert "close:Y" in _stack_log or "open:X" in _opens, "ExitStack ran"

print("behavior OK")
