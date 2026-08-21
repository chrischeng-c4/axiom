"""Surface contract for language scope (LEGB rule).

# type-regime: monomorphic

Probes: local, enclosing, global, builtin scope lookup; global/nonlocal
statements; variable shadowing; comprehension scope isolation.
CPython 3.12 is the oracle.
"""

# Local scope shadows global
_x = "global"
def _local_shadow() -> str:
    _x = "local"
    return _x

assert _local_shadow() == "local", f"local shadow = {_local_shadow()!r}"
assert _x == "global", f"global unchanged = {_x!r}"

# Global lookup when no local binding
_GLOBAL = 42
def _read_global() -> int:
    return _GLOBAL

assert _read_global() == 42, f"global read = {_read_global()!r}"

# global statement allows mutation
_counter = 0
def _inc_global() -> None:
    global _counter
    _counter += 1

_inc_global()
_inc_global()
assert _counter == 2, f"counter = {_counter!r}"

# nonlocal captures enclosing variable
def _make_enc():
    _enc = 0
    def _bump():
        nonlocal _enc
        _enc += 1
        return _enc
    return _bump

_bump = _make_enc()
assert _bump() == 1, f"enc1 = 1 expected"
assert _bump() == 2, f"enc2 = 2 expected"

# Builtin fallback
assert len([1, 2, 3]) == 3, "builtin len accessible"

# Comprehension has its own scope — iteration var doesn't leak
_result = [_i * 2 for _i in range(5)]
assert _result == [0, 2, 4, 6, 8], f"listcomp = {_result!r}"
_raised = False
try:
    _ = _i  # type: ignore[possibly-undefined]
except NameError:
    _raised = True
assert _raised, "comprehension iteration var should not leak"

# Class scope does NOT participate in enclosing lookup
_outer = "outer"
class _Cls:
    _outer = "class"  # shadows at class level only
    def _method(self) -> str:
        return _outer  # sees module-level, not class-level

assert _Cls()._method() == "outer", f"class scope = {_Cls()._method()!r}"

print("surface OK")
