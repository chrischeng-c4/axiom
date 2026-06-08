# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_io_namedtuple_random_silent"
# subject = "cpython321.lang_io_namedtuple_random_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_io_namedtuple_random_silent.py"
# status = "filled"
# ///
"""cpython321.lang_io_namedtuple_random_silent: execute CPython 3.12 seed lang_io_namedtuple_random_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across the
# core stdlib quintet pinned by atomic 157: `io` (the documented
# `StringIO` / `BytesIO` write+getvalue+read+readlines mutation
# protocol), `collections` (the documented `Counter.update` /
# `Counter.subtract` / `Counter.total` instance helpers + the
# documented `OrderedDict.move_to_end` reordering helper + the
# documented `namedtuple` instance indexing + `_asdict` /
# `_fields` / `_replace` instance helpers + the bare-class
# `__name__` identity on `defaultdict` / `deque` / `ChainMap`),
# and `random` (the documented `seed(42)` + `random()`
# deterministic Mersenne-Twister round-trip).
#
# The matching subset (collections.Counter("aabbcc"),
# defaultdict(list/int) accumulator, OrderedDict basic insertion
# order, namedtuple .x / .y attribute access, deque
# append / appendleft / pop / popleft / extend / extendleft /
# rotate, ChainMap first-map-wins, all itertools higher-order
# helpers chain / accumulate / takewhile / dropwhile / compress /
# starmap / groupby / product / permutations / combinations /
# combinations_with_replacement / zip_longest / filterfalse /
# tee, generator next / send / yield from, random range
# contracts) is covered by
# `test_collections_itertools_generator_value_ops`; this fixture
# pins the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • io.StringIO() — write then getvalue round-trip
#     "hello world" (mamba: getvalue returns "" — write is a
#     no-op);
#   • io.StringIO("abc").read() == "abc" (mamba: returns "");
#   • io.StringIO(text).readlines surface — bare hasattr
#     (mamba: AttributeError, 'dict' object has no attribute
#     'readlines');
#   • io.BytesIO() — write then getvalue round-trip b"hello"
#     (mamba: returns b"");
#   • io.BytesIO(b"abc").read() == b"abc" (mamba: returns b"");
#   • collections.Counter("a").total() == 1 — documented total
#     count helper (mamba: AttributeError, 'collections.Counter'
#     object has no attribute 'total');
#   • collections.Counter("ab").update("ab") == {"a": 2, "b": 2}
#     — duplicate-update increment (mamba: update is a no-op,
#     leaves counter at {"a": 1, "b": 1});
#   • collections.Counter("ab").subtract surface — documented
#     decrement helper (mamba: AttributeError);
#   • collections.OrderedDict.move_to_end("a") — documented
#     reordering helper (mamba: AttributeError);
#   • collections.namedtuple instance index access — Point(1, 2)
#     [0] == 1 (mamba: returns None);
#   • Point(1, 2)._asdict() == {"x": 1, "y": 2} — documented
#     instance-helper (mamba: AttributeError);
#   • Point._fields == ("x", "y") — documented class attribute
#     (mamba: AttributeError);
#   • Point(1, 2)._replace(x=10) == Point(10, 2) — documented
#     immutable update helper (mamba: AttributeError);
#   • collections.defaultdict.__name__ == "defaultdict" — bare
#     class identity (mamba: returns None);
#   • collections.deque.__name__ == "deque" (mamba: None);
#   • collections.ChainMap.__name__ == "ChainMap" (mamba: None);
#   • random.seed(42); random.random() == 0.6394267984578837 —
#     deterministic Mersenne-Twister contract (mamba: returns
#     0.3745401188473625, divergent seeded float).
import io as _io_mod
import collections as _collections_mod
import random as _random_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / module-level helpers / instance methods
# that mamba's bundled type stubs do not surface accurately.
io: Any = _io_mod
collections: Any = _collections_mod
random: Any = _random_mod


_ledger: list[int] = []

# 1) io.StringIO — write + getvalue + read round-trip
_sio = io.StringIO()
_sio.write("hello world")
assert _sio.getvalue() == "hello world"; _ledger.append(1)
assert io.StringIO("abc").read() == "abc"; _ledger.append(1)
assert hasattr(io.StringIO("a"), "readlines") == True; _ledger.append(1)

# 2) io.BytesIO — write + getvalue + read round-trip
_bio = io.BytesIO()
_bio.write(b"hello")
assert _bio.getvalue() == b"hello"; _ledger.append(1)
assert io.BytesIO(b"abc").read() == b"abc"; _ledger.append(1)

# 3) collections.Counter — total + update + subtract surface
assert collections.Counter("a").total() == 1; _ledger.append(1)
_ctr = collections.Counter("ab")
_ctr.update("ab")
assert dict(_ctr) == {"a": 2, "b": 2}; _ledger.append(1)
assert hasattr(collections.Counter("a"), "subtract") == True; _ledger.append(1)

# 4) collections.OrderedDict — move_to_end reordering
_od = collections.OrderedDict([("a", 1), ("b", 2), ("c", 3)])
_od.move_to_end("a")
assert list(_od.keys()) == ["b", "c", "a"]; _ledger.append(1)

# 5) collections.namedtuple — indexing + _asdict + _fields + _replace
_Pt = collections.namedtuple("Point", ["x", "y"])
_p = _Pt(1, 2)
assert _p[0] == 1; _ledger.append(1)
assert _p._asdict() == {"x": 1, "y": 2}; _ledger.append(1)
assert _Pt._fields == ("x", "y"); _ledger.append(1)
assert _p._replace(x=10) == _Pt(10, 2); _ledger.append(1)

# 6) collections — bare class __name__ identity
assert collections.defaultdict.__name__ == "defaultdict"; _ledger.append(1)
assert collections.deque.__name__ == "deque"; _ledger.append(1)
assert collections.ChainMap.__name__ == "ChainMap"; _ledger.append(1)

# 7) random — deterministic Mersenne-Twister seed contract
random.seed(42)
assert random.random() == 0.6394267984578837; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_io_namedtuple_random_silent {sum(_ledger)} asserts")
