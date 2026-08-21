"""Behavior contract for language scope (LEGB rule).

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: LEGB order — Local > Enclosing > Global > Builtin
_x = "global"
def _outer_scope():
    _x = "enclosing"
    def _inner_scope() -> str:
        return _x  # sees enclosing, not global
    return _inner_scope()

assert _outer_scope() == "enclosing", f"LEGB enclosing = {_outer_scope()!r}"

def _global_fallback() -> str:
    return _x  # sees global, no local

assert _global_fallback() == "global", f"LEGB global = {_global_fallback()!r}"

# Rule 2: Assigning without global/nonlocal creates local, hides outer
_g = 100
def _hides_global() -> int:
    _g = 999  # local, shadows module _g
    return _g

assert _hides_global() == 999, f"local = {_hides_global()!r}"
assert _g == 100, f"module _g unchanged = {_g!r}"

# Rule 3: UnboundLocalError — assigning in function makes variable local throughout
_raised = False
def _unboundlocal() -> None:
    try:
        _ = _val  # _val used before assignment → UnboundLocalError
    except UnboundLocalError:
        pass
    _val = 1  # assignment makes _val local to the whole function

_unboundlocal()  # should not raise

# Rule 4: global statement gives write access to module scope
_mod_var = 0
def _write_global() -> None:
    global _mod_var
    _mod_var = 7

_write_global()
assert _mod_var == 7, f"mod_var after write = {_mod_var!r}"

# Rule 5: nonlocal affects exactly one enclosing level
def _two_levels():
    _a = 1
    def _mid():
        _a = 2  # local to _mid, not nonlocal
        def _inner() -> int:
            nonlocal _a  # refers to _mid's _a
            _a += 10
            return _a
        return _inner()
    outer_a = _a
    _mid_result = _mid()
    return outer_a, _mid_result

_oa, _mr = _two_levels()
assert _oa == 1, f"outer a = {_oa!r}"
assert _mr == 12, f"mid result = {_mr!r}"

# Rule 6: Comprehension scope isolation (list/set/dict comp, genexpr)
_outer_i = 100
_lc = [_outer_i + j for j in range(3)]
assert _lc == [100, 101, 102], f"comp = {_lc!r}"
assert _outer_i == 100, f"outer_i = {_outer_i!r}"

# j should not exist after the comprehension
_raised2 = False
try:
    _ = j  # type: ignore[name-defined]
except NameError:
    _raised2 = True
assert _raised2, "comp iter var should not leak"

# Rule 7: Class body does NOT create enclosing scope for methods
_free = "module-level"
class _ScopeTest:
    _free = "class-level"
    def get_free(self) -> str:
        return _free  # sees module-level, not class-level

assert _ScopeTest().get_free() == "module-level", f"class scope isolation = {_ScopeTest().get_free()!r}"

# Rule 8: Builtin is the outermost scope
def _shadow_len():
    _len = lambda s: -1  # local shadows builtin
    return _len("hello"), len("hello")

_local_l, _builtin_l = _shadow_len()
assert _local_l == -1, f"local len = {_local_l!r}"
assert _builtin_l == 5, f"builtin len = {_builtin_l!r}"

print("behavior OK")
