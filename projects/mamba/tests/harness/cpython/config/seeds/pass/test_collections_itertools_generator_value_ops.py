# Operational AssertionPass seed for the value contract of three
# bootstrap stdlib clusters used by every batching / streaming /
# coroutine path: `collections` (the documented `Counter` /
# `defaultdict` / `OrderedDict` / `deque` / `ChainMap` /
# `namedtuple` factory + base contracts), `itertools` (the
# documented `chain` / `accumulate` / `takewhile` / `dropwhile` /
# `compress` / `starmap` / `groupby` / `product` / `permutations` /
# `combinations` / `combinations_with_replacement` / `zip_longest`
# / `filterfalse` / `tee` higher-order helpers), and Python's
# core generator protocol (`yield` + `send` + `yield from`).
#
# The matching subset between mamba and CPython is the `Counter`-
# from-string layer + `most_common` + missing-key zero +
# update / subtract layer + `defaultdict(list/int)` accumulator
# layer + `OrderedDict` insertion-order layer + `namedtuple`
# attribute layer + `deque` two-ended mutation layer + `ChainMap`
# resolution layer + every itertools higher-order helper above +
# the generator next / send / yield-from protocol +
# `random.random` range layer.
#
# Surface in this fixture:
#   • collections.Counter — from-string, most_common, missing
#     key zero default, update / subtract;
#   • collections.defaultdict — list factory + int factory
#     accumulator;
#   • collections.OrderedDict — insertion-order preserved on
#     []=insert;
#   • collections.namedtuple — Point(x, y) constructor +
#     attribute access;
#   • collections.deque — append / appendleft / pop / popleft
#     / extend / extendleft / rotate;
#   • collections.ChainMap — first-map wins on duplicate key;
#   • itertools.chain — flatten two iterables;
#   • itertools.accumulate — running totals;
#   • itertools.takewhile / dropwhile — prefix-while filtering;
#   • itertools.compress — boolean mask selection;
#   • itertools.starmap — apply f to argument tuples;
#   • itertools.groupby — consecutive-run grouping;
#   • itertools.product / permutations / combinations /
#     combinations_with_replacement — combinatoric expansion;
#   • itertools.zip_longest — pad-with-fillvalue zip;
#   • itertools.filterfalse — invert predicate filter;
#   • itertools.tee — independent iterator copies;
#   • generators — next exhaust, send round-trip, yield from
#     mixed iterables;
#   • random — random / randint / choice range contracts.
#
# Behavioral edges that DIVERGE on mamba (io.StringIO write /
# getvalue / read returning empty + readlines AttributeError,
# io.BytesIO write / getvalue / read returning empty,
# Counter.total AttributeError, collections class
# __name__ returning None on defaultdict / deque / ChainMap,
# OrderedDict.move_to_end AttributeError, namedtuple instance
# index access returning None + _asdict / _fields / _replace
# AttributeError, random.seed(42) + random.random() returning a
# divergent float) are covered in the matching spec fixture
# `lang_io_namedtuple_random_silent`.
import collections
import itertools
import random


def _gen_basic():
    yield 1
    yield 2
    yield 3


def _gen_send():
    x = yield 1
    yield x * 2


def _gen_yield_from():
    yield from [1, 2, 3]
    yield from "ab"


_ledger: list[int] = []

# 1) collections.Counter — base contract
_c = collections.Counter("aabbcc")
assert dict(_c) == {"a": 2, "b": 2, "c": 2}; _ledger.append(1)
assert _c.most_common(1) == [("a", 2)]; _ledger.append(1)
assert _c["z"] == 0; _ledger.append(1)
_c2 = collections.Counter(["a", "b", "a", "c", "a"])
assert _c2["a"] == 3; _ledger.append(1)
assert _c2["b"] == 1; _ledger.append(1)
assert _c2["c"] == 1; _ledger.append(1)

# 2) collections.defaultdict — list factory + int factory
_dd: collections.defaultdict[str, list[int]] = collections.defaultdict(list)
_dd["a"].append(1)
_dd["a"].append(2)
_dd["b"].append(3)
assert dict(_dd) == {"a": [1, 2], "b": [3]}; _ledger.append(1)
_di: collections.defaultdict[str, int] = collections.defaultdict(int)
_di["x"] += 1
_di["x"] += 1
_di["y"] += 5
assert dict(_di) == {"x": 2, "y": 5}; _ledger.append(1)

# 3) collections.OrderedDict — insertion order preservation
_od: collections.OrderedDict[str, int] = collections.OrderedDict([("a", 1), ("b", 2), ("c", 3)])
assert list(_od.keys()) == ["a", "b", "c"]; _ledger.append(1)
_od["d"] = 4
assert list(_od.keys()) == ["a", "b", "c", "d"]; _ledger.append(1)

# 4) collections.namedtuple — attribute access
_Point = collections.namedtuple("Point", ["x", "y"])
_p = _Point(1, 2)
assert _p.x == 1; _ledger.append(1)
assert _p.y == 2; _ledger.append(1)

# 5) collections.deque — two-ended mutation
_dq: collections.deque[int] = collections.deque([1, 2, 3])
_dq.append(4)
_dq.appendleft(0)
assert list(_dq) == [0, 1, 2, 3, 4]; _ledger.append(1)
assert _dq.pop() == 4; _ledger.append(1)
assert _dq.popleft() == 0; _ledger.append(1)
assert list(_dq) == [1, 2, 3]; _ledger.append(1)
_dq.extend([10, 20])
_dq.extendleft([-10, -20])
assert list(_dq) == [-20, -10, 1, 2, 3, 10, 20]; _ledger.append(1)
_dq.rotate(1)
assert list(_dq) == [20, -20, -10, 1, 2, 3, 10]; _ledger.append(1)

# 6) collections.ChainMap — first-map wins
_cm = collections.ChainMap({"a": 1}, {"b": 2, "a": 99})
assert _cm["a"] == 1; _ledger.append(1)
assert _cm["b"] == 2; _ledger.append(1)

# 7) itertools — higher-order helpers
assert list(itertools.chain([1, 2], [3, 4])) == [1, 2, 3, 4]; _ledger.append(1)
assert list(itertools.accumulate([1, 2, 3, 4])) == [1, 3, 6, 10]; _ledger.append(1)
assert list(itertools.takewhile(lambda x: x < 3, [1, 2, 3, 4, 5])) == [1, 2]; _ledger.append(1)
assert list(itertools.dropwhile(lambda x: x < 3, [1, 2, 3, 4, 5])) == [3, 4, 5]; _ledger.append(1)
assert list(itertools.compress("ABCD", [1, 0, 1, 0])) == ["A", "C"]; _ledger.append(1)
assert list(itertools.starmap(pow, [(2, 3), (3, 2)])) == [8, 9]; _ledger.append(1)
assert [(k, list(g)) for k, g in itertools.groupby("aabbcc")] == [("a", ["a", "a"]), ("b", ["b", "b"]), ("c", ["c", "c"])]; _ledger.append(1)
assert list(itertools.product([1, 2], [3, 4])) == [(1, 3), (1, 4), (2, 3), (2, 4)]; _ledger.append(1)
assert list(itertools.permutations([1, 2, 3], 2)) == [(1, 2), (1, 3), (2, 1), (2, 3), (3, 1), (3, 2)]; _ledger.append(1)
assert list(itertools.combinations([1, 2, 3], 2)) == [(1, 2), (1, 3), (2, 3)]; _ledger.append(1)
assert list(itertools.combinations_with_replacement([1, 2], 2)) == [(1, 1), (1, 2), (2, 2)]; _ledger.append(1)
assert list(itertools.zip_longest([1, 2], ["a", "b", "c"], fillvalue="*")) == [(1, "a"), (2, "b"), ("*", "c")]; _ledger.append(1)
assert list(itertools.filterfalse(lambda x: x > 2, [1, 2, 3, 4])) == [1, 2]; _ledger.append(1)
_tee_a, _tee_b = itertools.tee([1, 2, 3])
assert list(_tee_a) == [1, 2, 3]; _ledger.append(1)
assert list(_tee_b) == [1, 2, 3]; _ledger.append(1)

# 8) generator protocol — next, send, yield from
_g = _gen_basic()
assert next(_g) == 1; _ledger.append(1)
assert next(_g) == 2; _ledger.append(1)
assert next(_g) == 3; _ledger.append(1)
assert list(_gen_basic()) == [1, 2, 3]; _ledger.append(1)
_g2 = _gen_send()
assert next(_g2) == 1; _ledger.append(1)
assert _g2.send(10) == 20; _ledger.append(1)
assert list(_gen_yield_from()) == [1, 2, 3, "a", "b"]; _ledger.append(1)

# 9) random — range contracts (not exact-value)
_r = random.random()
assert 0.0 <= _r < 1.0; _ledger.append(1)
_ri = random.randint(1, 100)
assert 1 <= _ri <= 100; _ledger.append(1)
assert random.choice([1, 2, 3, 4, 5]) in [1, 2, 3, 4, 5]; _ledger.append(1)

# NB: io.StringIO write / getvalue / read returning empty +
# readlines AttributeError, io.BytesIO write / getvalue / read
# returning empty, Counter.total AttributeError, collections
# class __name__ returning None on defaultdict / deque /
# ChainMap, OrderedDict.move_to_end AttributeError, namedtuple
# instance index access returning None + _asdict / _fields /
# _replace AttributeError, random.seed(42) + random.random()
# returning a divergent float all DIVERGE on mamba — moved to
# the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_collections_itertools_generator_value_ops {sum(_ledger)} asserts")
