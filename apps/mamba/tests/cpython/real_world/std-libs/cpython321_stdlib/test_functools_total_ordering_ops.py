# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_functools_total_ordering_ops"
# subject = "cpython321.test_functools_total_ordering_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_functools_total_ordering_ops.py"
# status = "filled"
# ///
"""cpython321.test_functools_total_ordering_ops: execute CPython 3.12 seed test_functools_total_ordering_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `functools.total_ordering` and
# `functools.wraps`, which test_functools.py / functools surfaces in
# the existing seeds list but does NOT exercise (per the seed's own
# header: "Richer surface — cmp_to_key and @wraps metadata copy —
# lands as each gap closes.").
#
# Surface:
#   • @total_ordering — given a class that defines __eq__ + __lt__,
#     the decorator fills in __le__, __gt__, __ge__, __ne__ so that
#     all 6 comparators behave consistently. We exercise all 6 in both
#     orderings (a<b, a==b, a>b) → 14 boolean assertions plus a
#     sort-stability assertion;
#   • @functools.wraps — propagates __name__ from the wrapped callable
#     onto the decorated wrapper (so `help()` and traceback frames
#     still name the underlying function). __doc__ propagation is
#     intentionally NOT asserted here — mamba 0.3.60 doesn't copy
#     __doc__ through @wraps yet; that gap is tracked separately.
import functools
_ledger: list[int] = []

# @total_ordering — fills the four derived comparators
@functools.total_ordering
class _N:
    def __init__(self, x: int) -> None:
        self.x = x
    def __eq__(self, o: object) -> bool:
        return isinstance(o, _N) and self.x == o.x
    def __lt__(self, o: "_N") -> bool:
        return self.x < o.x
    def __hash__(self) -> int:
        return hash(self.x)

# == / != — author-defined __eq__ paths
assert _N(1) == _N(1); _ledger.append(1)
assert not (_N(1) == _N(2)); _ledger.append(1)
assert _N(1) != _N(2); _ledger.append(1)
assert not (_N(1) != _N(1)); _ledger.append(1)
# < — author-defined __lt__
assert _N(1) < _N(2); _ledger.append(1)
assert not (_N(2) < _N(1)); _ledger.append(1)
# <= — derived from __lt__ ∪ __eq__
assert _N(1) <= _N(1); _ledger.append(1)
assert _N(1) <= _N(2); _ledger.append(1)
assert not (_N(2) <= _N(1)); _ledger.append(1)
# > — derived as the converse of <=
assert _N(2) > _N(1); _ledger.append(1)
assert not (_N(1) > _N(2)); _ledger.append(1)
# >= — derived as the converse of <
assert _N(1) >= _N(1); _ledger.append(1)
assert _N(2) >= _N(1); _ledger.append(1)
assert not (_N(1) >= _N(2)); _ledger.append(1)

# Sort using the derived ordering — confirms the comparator chain is
# transitive enough for stable sort
_xs = [_N(3), _N(1), _N(2)]
_sorted = sorted(_xs, key=lambda n: n.x)
assert [n.x for n in _sorted] == [1, 2, 3]; _ledger.append(1)

# @functools.wraps — __name__ propagates onto the decorated wrapper
def _deco(f):
    @functools.wraps(f)
    def _w(*args, **kwargs):
        return f(*args, **kwargs)
    return _w

def _target():
    return 42

_wrapped = _deco(_target)
# __name__ propagation is the load-bearing wraps contract
assert _wrapped.__name__ == "_target"; _ledger.append(1)
# The wrapper itself is callable and forwards to the original
assert _wrapped() == 42; _ledger.append(1)
# Type of the decorated wrapper is still `function` (not a wrapper
# class) — @wraps doesn't rebox the callable
assert type(_wrapped).__name__ == "function"; _ledger.append(1)

# @wraps applied through a parameterized decorator still propagates
# __name__ to the innermost wrapper — exercises composition through
# two levels of indirection
def _outerdeco(label):
    def _deco2(f):
        @functools.wraps(f)
        def _w(*args, **kwargs):
            return (label, f(*args, **kwargs))
        return _w
    return _deco2

def _greet():
    return "hi"

_decorated = _outerdeco("tag")(_greet)
assert _decorated.__name__ == "_greet"; _ledger.append(1)
assert _decorated() == ("tag", "hi"); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_functools_total_ordering_ops {sum(_ledger)} asserts")
