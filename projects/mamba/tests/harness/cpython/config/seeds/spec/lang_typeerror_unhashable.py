# Spec seed for CPython TypeError contract on unhashable-type usage.
# Surface: CPython raises TypeError("unhashable type: '<name>'") when
# a list / dict / set / bytearray is used in any context that requires
# a hashable key:
#   • set([list-of-unhashable]) / frozenset(...);
#   • set.add(unhashable) / .discard / .remove;
#   • dict[unhashable] = v / dict.setdefault / dict.__contains__;
#   • set.__contains__ with unhashable probe;
#   • hash() called directly;
#   • a tuple containing an unhashable element (tuple itself is
#     hashable only if every component is).
#
# Mamba 0.3.60 currently DOES NOT raise TypeError on any of these
# forms; the unhashable key flows through to the underlying hashmap
# silently (one form — set.remove(unhashable) — raises KeyError on
# mamba, also a divergence from CPython's TypeError). This seed pins
# Fail today so the runner surfaces drift when mamba grows hashable
# enforcement.
#
# `Any`-typed holders keep static type-checkers (Pyright) from
# flagging the intentional unhashable-key use before runtime.
from typing import Any
_ledger: list[int] = []

_lst: Any = [1, 2]
_dct: Any = {"a": 1}
_st: Any = {1, 2}
_ba: Any = bytearray(b"abc")

# set([unhashable])
try:
    _ = set([_lst])
    raise AssertionError("set with unhashable element must raise TypeError")
except TypeError:
    _ledger.append(1)

# frozenset([unhashable])
try:
    _ = frozenset([_lst])
    raise AssertionError("frozenset with unhashable element must raise TypeError")
except TypeError:
    _ledger.append(1)

# set.add(unhashable)
_s_add: Any = set()
try:
    _s_add.add(_lst)
    raise AssertionError("set.add(list) must raise TypeError")
except TypeError:
    _ledger.append(1)

# set.discard(unhashable)
_s_disc: Any = {1, 2, 3}
try:
    _s_disc.discard(_lst)
    raise AssertionError("set.discard(list) must raise TypeError")
except TypeError:
    _ledger.append(1)

# set.remove(unhashable) — CPython raises TypeError (hash check fires
# before the membership check); mamba may raise KeyError instead
_s_rem: Any = {1, 2, 3}
try:
    _s_rem.remove(_lst)
    raise AssertionError("set.remove(list) must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict[unhashable] = v — assignment
_d_assign: Any = {}
try:
    _d_assign[_lst] = 1
    raise AssertionError("dict[list] = v must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict[unhashable_dict] = v
_d_assign2: Any = {}
try:
    _d_assign2[_dct] = 1
    raise AssertionError("dict[dict] = v must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict[unhashable_set] = v
_d_assign3: Any = {}
try:
    _d_assign3[_st] = 1
    raise AssertionError("dict[set] = v must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict.setdefault(unhashable, default)
_d_sd: Any = {}
try:
    _d_sd.setdefault(_lst, 99)
    raise AssertionError("dict.setdefault(list, ...) must raise TypeError")
except TypeError:
    _ledger.append(1)

# `unhashable in dict` — membership probe
_d_in: Any = {"a": 1}
try:
    _ = _lst in _d_in
    raise AssertionError("list in dict must raise TypeError")
except TypeError:
    _ledger.append(1)

# `unhashable in set` — membership probe
_s_in: Any = {1, 2}
try:
    _ = _lst in _s_in
    raise AssertionError("list in set must raise TypeError")
except TypeError:
    _ledger.append(1)

# hash(list)
try:
    _ = hash(_lst)
    raise AssertionError("hash(list) must raise TypeError")
except TypeError:
    _ledger.append(1)

# hash(dict)
try:
    _ = hash(_dct)
    raise AssertionError("hash(dict) must raise TypeError")
except TypeError:
    _ledger.append(1)

# hash(set)
try:
    _ = hash(_st)
    raise AssertionError("hash(set) must raise TypeError")
except TypeError:
    _ledger.append(1)

# hash(bytearray)
try:
    _ = hash(_ba)
    raise AssertionError("hash(bytearray) must raise TypeError")
except TypeError:
    _ledger.append(1)

# tuple-of-unhashable as dict key — tuple is hashable only if every
# component is; (1, [2]) trips the recursive check
_d_tup: Any = {}
_tup_unh: Any = (1, [2])
try:
    _d_tup[_tup_unh] = 1
    raise AssertionError("dict[tuple-with-list] must raise TypeError")
except TypeError:
    _ledger.append(1)

# tuple-of-unhashable as set element
_s_tup: Any = set()
try:
    _s_tup.add(_tup_unh)
    raise AssertionError("set.add(tuple-with-list) must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict comprehension with unhashable key
try:
    _ = {_lst: 1}  # type: ignore
    raise AssertionError("{list: 1} literal must raise TypeError")
except TypeError:
    _ledger.append(1)

# set literal with unhashable element
try:
    _ = {_lst}  # type: ignore
    raise AssertionError("{list} literal must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typeerror_unhashable {sum(_ledger)} asserts")
