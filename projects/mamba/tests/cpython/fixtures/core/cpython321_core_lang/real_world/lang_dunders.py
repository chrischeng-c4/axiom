# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_dunders"
# subject = "cpython321.lang_dunders"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_dunders.py"
# status = "filled"
# ///
"""cpython321.lang_dunders: execute CPython 3.12 seed lang_dunders"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# lang_dunders.py — #3360 axis-1 full dunder set seed.
#
# Exercises the four dunder families enumerated in the issue body:
#   1. Numeric: __add__, __sub__, __mul__
#   2. Comparison: __eq__, __lt__, __gt__
#   3. Container: __getitem__, __setitem__, __len__, __contains__
#   4. Reflected: __radd__ (called when LHS has no __add__)
#
# Mamba quirks (tracked separately):
#   * __format__ ignored by format() and __delitem__ not invoked by del (#3504).
#   * Operator-overload bypass for native ints — int-handle pattern carve-out.
#     Not exercised here; the classes wrap user-defined fields explicitly.
#
# Contract: AssertionError → Fail; MAMBA_ASSERTION_PASS → AssertionPass.

_ledger: list[int] = []

# (1) Numeric dunders: +, -, *
class _N:
    def __init__(self, v):
        self.v = v
    def __add__(self, other):
        return _N(self.v + (other.v if isinstance(other, _N) else other))
    def __sub__(self, other):
        return _N(self.v - (other.v if isinstance(other, _N) else other))
    def __mul__(self, other):
        return _N(self.v * (other.v if isinstance(other, _N) else other))

_a = _N(3)
_b = _N(4)
_c = _a + _b
assert _c.v - 7 == 0, f"_N.__add__ produces _N(7), got {_c.v!r}"
_ledger.append(1)

_diff = _a - _N(1)
assert _diff.v - 2 == 0, f"_N.__sub__ produces _N(2), got {_diff.v!r}"
_ledger.append(1)

_prod = _a * 5
assert _prod.v - 15 == 0, f"_N.__mul__ with int arg produces _N(15), got {_prod.v!r}"
_ledger.append(1)

# (2) Comparison dunders: ==, <, >
class _Cmp:
    def __init__(self, v):
        self.v = v
    def __eq__(self, other):
        return isinstance(other, _Cmp) and self.v == other.v
    def __lt__(self, other):
        return self.v < other.v
    def __gt__(self, other):
        return self.v > other.v

assert _Cmp(1) == _Cmp(1), "_Cmp(1) == _Cmp(1) via __eq__"
_ledger.append(1)

assert not (_Cmp(1) == _Cmp(2)), "_Cmp(1) != _Cmp(2)"
_ledger.append(1)

assert _Cmp(1) < _Cmp(2), "_Cmp(1) < _Cmp(2) via __lt__"
_ledger.append(1)

assert _Cmp(3) > _Cmp(2), "_Cmp(3) > _Cmp(2) via __gt__"
_ledger.append(1)

# (3) Container dunders: getitem / setitem / len / contains
class _Cont:
    def __init__(self):
        self._data = {"a": 1, "b": 2}
    def __getitem__(self, k):
        return self._data[k]
    def __setitem__(self, k, v):
        self._data[k] = v
    def __len__(self):
        return len(self._data)
    def __contains__(self, k):
        return k in self._data

_ct = _Cont()
assert _ct["a"] - 1 == 0, f"_Cont.__getitem__('a') == 1, got {_ct['a']!r}"
_ledger.append(1)

_ct["c"] = 3
assert _ct["c"] - 3 == 0, f"_Cont.__setitem__ reflected, got {_ct['c']!r}"
_ledger.append(1)

assert len(_ct) - 3 == 0, f"_Cont.__len__ returns 3 after one insert, got {len(_ct)!r}"
_ledger.append(1)

assert "a" in _ct, "_Cont.__contains__ True for 'a'"
_ledger.append(1)

assert "z" not in _ct, "_Cont.__contains__ False for 'z'"
_ledger.append(1)

# (4) Reflected dunder: __radd__ invoked when LHS doesn't know about RHS
class _R:
    def __init__(self, v):
        self.v = v
    def __radd__(self, other):
        return ("radd", other, self.v)

_r = _R(10)
_out = 5 + _r
assert _out == ("radd", 5, 10), (
    f"int + _R triggers __radd__, got {_out!r}"
)
_ledger.append(1)

# (5) Chain of operators: ((_N(2) + _N(3)) * _N(4)).v == 20
_chained = (_N(2) + _N(3)) * _N(4)
assert _chained.v - 20 == 0, (
    f"chained ((2+3)*4) via dunders, got {_chained.v!r}"
)
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_dunders {sum(_ledger)} asserts")
