# Operational AssertionPass seed for SILENT divergences across the
# enum metaclass + Enum instance value-truth + collections module
# extended identifier surface + collections.OrderedDict mutating
# helpers + collections.ChainMap class identity + collections.deque
# stored-length / index value contract + collections.namedtuple
# extended tuple-subscript / _fields / _asdict / len surface +
# collections.abc full hasattr surface pinned by atomic 181: `enum`
# (the documented `EnumMeta` class identifier + the documented
# `Enum` instance .name / .value attribute-binding contract + the
# documented enum-class iteration contract + the documented enum-
# class value-lookup contract), `collections` (the documented
# `type(OrderedDict()).__name__` / `type(defaultdict(int)).__name__`
# / `type(Counter()).__name__` / `type(ChainMap()).__name__`
# class-identity contracts + the documented `move_to_end` /
# `popitem` mutating-helper contract on OrderedDict + the documented
# `len(deque)` stored-length / `deque[index]` index value contract +
# the documented `namedtuple` extended tuple-subscript / `_fields` /
# `_asdict()` / `len(...)` contract), and `collections.abc` (the
# documented `Iterable` / `Iterator` / `Container` / `Hashable` /
# `Sized` / `Callable` / `Sequence` / `MutableSequence` / `Mapping`
# / `MutableMapping` / `Set` / `MutableSet` class identifiers).
#
# The matching subset (partial enum module hasattr surface (Enum /
# IntEnum / Flag / IntFlag / auto / unique), full collections
# module hasattr surface + partial OrderedDict / defaultdict /
# Counter instance basic-value contract + namedtuple attribute
# access, full queue / weakref module hasattr surface, and the
# heapq value contract) is covered by
# `test_collections_queue_weakref_heapq_hasattr_ops`; this fixture
# pins the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • hasattr(enum, "EnumMeta") is True — documented class
#     identifier (mamba: False);
#   • Color.RED.name == "RED" — documented Enum instance
#     attribute-binding contract (mamba: returns None);
#   • Color.RED.value == 1 — documented Enum instance value
#     attribute-binding contract (mamba: returns None);
#   • type(OrderedDict()).__name__ == "OrderedDict" —
#     documented class-identity contract (mamba: returns
#     "collections.OrderedDict" — the bare module-qualified
#     name leaks);
#   • type(defaultdict(int)).__name__ == "defaultdict" —
#     documented class-identity contract (mamba: returns
#     "collections.defaultdict");
#   • type(Counter()).__name__ == "Counter" — documented
#     class-identity contract (mamba: returns
#     "collections.Counter");
#   • type(ChainMap()).__name__ == "ChainMap" — documented
#     class-identity contract (mamba: returns "dict" — the
#     ChainMap class is a plain dict alias);
#   • OrderedDict.move_to_end works — documented mutating
#     helper (mamba: raises AttributeError);
#   • OrderedDict.popitem works — documented mutating helper
#     (mamba: raises AttributeError);
#   • len(deque([1,2,3,4])) == 4 — documented stored-length
#     contract (mamba: returns 0 — the deque instance is an
#     opaque handle that does not expose the inner length);
#   • deque([10,20,30])[0] == 10 — documented index value
#     contract (mamba: returns None);
#   • Point[0] == 1 — documented namedtuple tuple-subscript
#     contract (mamba: returns None);
#   • Point._fields == ("x", "y") — documented namedtuple
#     introspection contract (mamba: returns None);
#   • Point._asdict() == {"x": 1, "y": 2} — documented
#     namedtuple introspection contract (mamba: raises
#     AttributeError);
#   • len(Point) == 2 — documented namedtuple tuple-length
#     contract (mamba: raises or returns None);
#   • hasattr(collections.abc, "Iterable") and the rest of
#     the documented `collections.abc` class identifiers —
#     when imported via `from collections import abc`, the
#     full Iterable / Iterator / Container / Hashable /
#     Sized / Callable / Sequence / MutableSequence /
#     Mapping / MutableMapping / Set / MutableSet identifier
#     surface (mamba: the bound `abc` proxy is empty —
#     every documented identifier is missing).
import enum as _enum_mod
import collections as _collections_mod
from collections import abc as _cabc_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / instance attribute-binding / mutating-helper /
# tuple-subscript behavior that mamba's bundled type stubs do not
# surface accurately.
enum: Any = _enum_mod
collections: Any = _collections_mod
cabc: Any = _cabc_mod


class _Color(enum.Enum):
    RED = 1
    GREEN = 2
    BLUE = 3


_Point = collections.namedtuple('_Point', ['x', 'y'])


_ledger: list[int] = []

# 1) enum — EnumMeta class identifier + Enum instance
#    .name / .value
assert hasattr(enum, "EnumMeta") == True; _ledger.append(1)
assert _Color.RED.name == "RED"; _ledger.append(1)
assert _Color.RED.value == 1; _ledger.append(1)
assert _Color.GREEN.name == "GREEN"; _ledger.append(1)
assert _Color.GREEN.value == 2; _ledger.append(1)

# 2) collections — type identity (class-name leak)
assert type(collections.OrderedDict()).__name__ == "OrderedDict"; _ledger.append(1)
assert type(collections.defaultdict(int)).__name__ == "defaultdict"; _ledger.append(1)
assert type(collections.Counter()).__name__ == "Counter"; _ledger.append(1)
assert type(collections.ChainMap()).__name__ == "ChainMap"; _ledger.append(1)

# 3) OrderedDict — move_to_end + popitem
_od = collections.OrderedDict()
_od['a'] = 1
_od['b'] = 2
_od['c'] = 3
_od.move_to_end('a')
assert list(_od.keys()) == ['b', 'c', 'a']; _ledger.append(1)

_od2 = collections.OrderedDict()
_od2['x'] = 99
_popped = _od2.popitem()
assert _popped == ('x', 99); _ledger.append(1)

# 4) deque — stored-length + index value
_dq = collections.deque([1, 2, 3, 4])
assert len(_dq) == 4; _ledger.append(1)
assert _dq[0] == 1; _ledger.append(1)
assert _dq[-1] == 4; _ledger.append(1)

_dq2 = collections.deque([10, 20, 30])
assert len(_dq2) == 3; _ledger.append(1)
assert _dq2[0] == 10; _ledger.append(1)

# 5) namedtuple — tuple-subscript + _fields + _asdict + len
_p = _Point(1, 2)
assert _p[0] == 1; _ledger.append(1)
assert _p[1] == 2; _ledger.append(1)
assert _p._fields == ("x", "y"); _ledger.append(1)
assert _p._asdict() == {"x": 1, "y": 2}; _ledger.append(1)
assert len(_p) == 2; _ledger.append(1)

# 6) collections.abc — full hasattr surface
assert hasattr(cabc, "Iterable") == True; _ledger.append(1)
assert hasattr(cabc, "Iterator") == True; _ledger.append(1)
assert hasattr(cabc, "Container") == True; _ledger.append(1)
assert hasattr(cabc, "Hashable") == True; _ledger.append(1)
assert hasattr(cabc, "Sized") == True; _ledger.append(1)
assert hasattr(cabc, "Callable") == True; _ledger.append(1)
assert hasattr(cabc, "Sequence") == True; _ledger.append(1)
assert hasattr(cabc, "MutableSequence") == True; _ledger.append(1)
assert hasattr(cabc, "Mapping") == True; _ledger.append(1)
assert hasattr(cabc, "MutableMapping") == True; _ledger.append(1)
assert hasattr(cabc, "Set") == True; _ledger.append(1)
assert hasattr(cabc, "MutableSet") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_enum_orderdict_deque_namedtuple_silent {sum(_ledger)} asserts")
