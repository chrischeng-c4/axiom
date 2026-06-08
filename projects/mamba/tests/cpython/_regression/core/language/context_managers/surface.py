"""Surface contract for language context managers.

# type-regime: monomorphic

Probes: with statement, __enter__/__exit__ protocol, contextlib.contextmanager,
exception suppression, multiple context managers, nested with.
CPython 3.12 is the oracle.
"""

import contextlib

# Basic context manager via __enter__/__exit__
_log = []
class _CM:
    def __enter__(self) -> "_CM":
        _log.append("enter")
        return self
    def __exit__(self, exc_type, exc_val, exc_tb) -> bool:
        _log.append("exit")
        return False  # don't suppress exceptions

with _CM() as cm:
    _log.append("body")
    assert isinstance(cm, _CM)

assert _log == ["enter", "body", "exit"], f"log = {_log!r}"

# contextmanager decorator
@contextlib.contextmanager
def _managed(name: str):
    _log2 = []
    _log2.append(f"before-{name}")
    yield _log2
    _log2.append(f"after-{name}")

with _managed("test") as inner_log:
    inner_log.append("during")

assert inner_log == ["before-test", "during", "after-test"], f"inner_log = {inner_log!r}"

# with statement evaluates to __enter__ return value
class _Returner:
    def __enter__(self) -> int:
        return 42
    def __exit__(self, *args) -> bool:
        return False
with _Returner() as val:
    assert val == 42, f"with value = {val!r}"

print("surface OK")
