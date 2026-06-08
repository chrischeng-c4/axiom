# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_collections_queue_weakref_heapq_hasattr_ops"
# subject = "cpython321.test_collections_queue_weakref_heapq_hasattr_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_collections_queue_weakref_heapq_hasattr_ops.py"
# status = "filled"
# ///
"""cpython321.test_collections_queue_weakref_heapq_hasattr_ops: execute CPython 3.12 seed test_collections_queue_weakref_heapq_hasattr_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `enum` / `collections` / `collections.abc` / `queue` /
# `weakref` / `heapq` six-pack pinned to atomic 181: `enum`
# (the documented partial module-level helper hasattr surface —
# `Enum` / `IntEnum` / `Flag` / `IntFlag` / `auto` / `unique`),
# `collections` (the documented full module-level helper
# hasattr surface — `OrderedDict` / `defaultdict` / `Counter`
# / `deque` / `namedtuple` / `ChainMap` / `UserDict` /
# `UserList` / `UserString` + the documented `OrderedDict` /
# `defaultdict` / `Counter` / `namedtuple` instance basic-
# value contract), `collections.abc` (the documented full
# module-level helper hasattr surface — `Iterable` /
# `Iterator` / `Container` / `Hashable` / `Sized` /
# `Callable` / `Sequence` / `MutableSequence` / `Mapping` /
# `MutableMapping` / `Set` / `MutableSet`), `queue` (the
# documented full module-level helper hasattr surface —
# `Queue` / `LifoQueue` / `PriorityQueue` / `SimpleQueue` /
# `Empty` / `Full`), `weakref` (the documented full module-
# level helper hasattr surface — `ref` /
# `WeakValueDictionary` / `WeakKeyDictionary` / `WeakSet` /
# `proxy` / `finalize`), and `heapq` (the documented
# `heapify` / `heappush` / `heappop` value contract).
#
# The matching subset between mamba and CPython is the
# partial `enum` module hasattr surface (`Enum` / `IntEnum`
# / `Flag` / `IntFlag` / `auto` / `unique` — `EnumMeta`
# DIVERGES + Enum instance .name / .value / iteration /
# value-lookup DIVERGE), the full `collections` module
# hasattr surface, the `OrderedDict` / `defaultdict` /
# `Counter` instance basic-value layer (insert + key access,
# list-of-keys, += accumulator, most_common, list-source
# counting) + `namedtuple` instance attribute-access layer
# (.x / .y) — `OrderedDict.move_to_end` /
# `OrderedDict.popitem` / `type(OrderedDict()).__name__` /
# `type(defaultdict(int)).__name__` /
# `type(Counter()).__name__` / `type(ChainMap()).__name__`
# / `len(deque)` / `deque[0]` / `Point[0]` / `Point._fields`
# / `Point._asdict()` / `len(Point)` / the full
# `collections.abc` hasattr surface all DIVERGE — moved to
# spec fixture, the full `queue` module hasattr surface,
# the full `weakref` module hasattr surface, and the
# `heapq` heapify / heappush / heappop value contract.
#
# Surface in this fixture:
#   • enum — partial module hasattr surface (Enum / IntEnum
#     / Flag / IntFlag / auto / unique);
#   • collections — full module hasattr surface
#     (OrderedDict / defaultdict / Counter / deque /
#     namedtuple / ChainMap / UserDict / UserList /
#     UserString);
#   • OrderedDict — insert + key access + list(keys());
#   • defaultdict — list factory + int factory accumulator;
#   • Counter — char count + most_common(2) + list source;
#   • namedtuple — instance attribute access (.x / .y);
#   • queue — full module hasattr surface (Queue /
#     LifoQueue / PriorityQueue / SimpleQueue / Empty /
#     Full);
#   • weakref — full module hasattr surface (ref /
#     WeakValueDictionary / WeakKeyDictionary / WeakSet /
#     proxy / finalize);
#   • heapq — heapify / heappush / heappop value contract.
#
# Behavioral edges that DIVERGE on mamba
# (hasattr(enum, "EnumMeta") False, Color.RED.name returns
# None, Color.RED.value returns None, list(Color) iteration
# returns [None, None, None] not the member list,
# type(OrderedDict()).__name__ returns "collections.OrderedDict"
# not "OrderedDict", type(defaultdict(int)).__name__ returns
# "collections.defaultdict" not "defaultdict",
# type(Counter()).__name__ returns "collections.Counter" not
# "Counter", type(ChainMap()).__name__ returns "dict" not
# "ChainMap", OrderedDict.move_to_end raises AttributeError,
# OrderedDict.popitem raises AttributeError, len(deque(...))
# returns 0 not the documented length, deque[0] returns None,
# namedtuple Point[0] returns None, Point._fields returns
# None, Point._asdict raises AttributeError, len(Point)
# fails) are covered in the matching spec fixture
# `lang_enum_orderdict_deque_namedtuple_silent`.
import enum
import collections
import queue
import weakref
import heapq


_ledger: list[int] = []

# 1) enum — module hasattr surface (EnumMeta DIVERGES — moved
#    to spec fixture)
assert hasattr(enum, "Enum") == True; _ledger.append(1)
assert hasattr(enum, "IntEnum") == True; _ledger.append(1)
assert hasattr(enum, "Flag") == True; _ledger.append(1)
assert hasattr(enum, "IntFlag") == True; _ledger.append(1)
assert hasattr(enum, "auto") == True; _ledger.append(1)
assert hasattr(enum, "unique") == True; _ledger.append(1)

# 2) collections — full module hasattr surface
assert hasattr(collections, "OrderedDict") == True; _ledger.append(1)
assert hasattr(collections, "defaultdict") == True; _ledger.append(1)
assert hasattr(collections, "Counter") == True; _ledger.append(1)
assert hasattr(collections, "deque") == True; _ledger.append(1)
assert hasattr(collections, "namedtuple") == True; _ledger.append(1)
assert hasattr(collections, "ChainMap") == True; _ledger.append(1)
assert hasattr(collections, "UserDict") == True; _ledger.append(1)
assert hasattr(collections, "UserList") == True; _ledger.append(1)
assert hasattr(collections, "UserString") == True; _ledger.append(1)

# 3) OrderedDict — insert + key access + list(keys())
_od = collections.OrderedDict()
_od['a'] = 1
_od['b'] = 2
_od['c'] = 3
assert _od['a'] == 1; _ledger.append(1)
assert _od['b'] == 2; _ledger.append(1)
assert _od['c'] == 3; _ledger.append(1)
assert list(_od.keys()) == ['a', 'b', 'c']; _ledger.append(1)

# 4) defaultdict — list factory + int factory accumulator
_dd_list = collections.defaultdict(list)
_dd_list['x'].append(1)
_dd_list['x'].append(2)
assert _dd_list['x'] == [1, 2]; _ledger.append(1)

_dd_int = collections.defaultdict(int)
_dd_int['a'] += 1
_dd_int['a'] += 1
_dd_int['b'] += 1
assert _dd_int['a'] == 2; _ledger.append(1)
assert _dd_int['b'] == 1; _ledger.append(1)

# 5) Counter — char count + most_common(2) + list source
_c = collections.Counter("aabbbcccc")
assert _c['a'] == 2; _ledger.append(1)
assert _c['b'] == 3; _ledger.append(1)
assert _c['c'] == 4; _ledger.append(1)
assert _c.most_common(2) == [('c', 4), ('b', 3)]; _ledger.append(1)

_c2 = collections.Counter([1, 2, 2, 3, 3, 3])
assert _c2[1] == 1; _ledger.append(1)
assert _c2[2] == 2; _ledger.append(1)
assert _c2[3] == 3; _ledger.append(1)

# 6) namedtuple — instance attribute access (.x / .y)
_Point = collections.namedtuple('_Point', ['x', 'y'])
_p = _Point(1, 2)
assert _p.x == 1; _ledger.append(1)
assert _p.y == 2; _ledger.append(1)

_p2 = _Point(10, 20)
assert _p2.x == 10; _ledger.append(1)
assert _p2.y == 20; _ledger.append(1)

# 7) queue — full module hasattr surface
assert hasattr(queue, "Queue") == True; _ledger.append(1)
assert hasattr(queue, "LifoQueue") == True; _ledger.append(1)
assert hasattr(queue, "PriorityQueue") == True; _ledger.append(1)
assert hasattr(queue, "SimpleQueue") == True; _ledger.append(1)
assert hasattr(queue, "Empty") == True; _ledger.append(1)
assert hasattr(queue, "Full") == True; _ledger.append(1)

# 8) weakref — full module hasattr surface
assert hasattr(weakref, "ref") == True; _ledger.append(1)
assert hasattr(weakref, "WeakValueDictionary") == True; _ledger.append(1)
assert hasattr(weakref, "WeakKeyDictionary") == True; _ledger.append(1)
assert hasattr(weakref, "WeakSet") == True; _ledger.append(1)
assert hasattr(weakref, "proxy") == True; _ledger.append(1)
assert hasattr(weakref, "finalize") == True; _ledger.append(1)

# 9) heapq — heapify / heappush / heappop value contract
_h = [3, 1, 4, 1, 5, 9, 2, 6]
heapq.heapify(_h)
assert _h[0] == 1; _ledger.append(1)
heapq.heappush(_h, 0)
assert _h[0] == 0; _ledger.append(1)
_popped = heapq.heappop(_h)
assert _popped == 0; _ledger.append(1)
assert _h[0] == 1; _ledger.append(1)

# NB: hasattr(enum, "EnumMeta") False on mamba, Color.RED.name
# / .value return None, list(Color) returns [None, None, ...],
# type(OrderedDict()).__name__ is "collections.OrderedDict" not
# "OrderedDict", type(defaultdict(int)).__name__ is
# "collections.defaultdict" not "defaultdict",
# type(Counter()).__name__ is "collections.Counter" not
# "Counter", type(ChainMap()).__name__ is "dict" not
# "ChainMap", OrderedDict.move_to_end raises AttributeError,
# OrderedDict.popitem raises AttributeError, len(deque(...))
# returns 0, deque[0] returns None, namedtuple Point[0] returns
# None, Point._fields returns None, Point._asdict raises
# AttributeError — all DIVERGE on mamba — moved to the
# divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_collections_queue_weakref_heapq_hasattr_ops {sum(_ledger)} asserts")
