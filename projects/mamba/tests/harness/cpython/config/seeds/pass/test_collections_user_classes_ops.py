# Operational AssertionPass seed for the three `collections`
# subclass-friendly wrappers — `UserDict`, `UserList`, `UserString`.
# Existing `collections` seeds (test_collections, test_collections_deque_ops,
# test_counter_ops, test_counter_extras_ops) cover Counter / deque /
# ChainMap / defaultdict / OrderedDict / namedtuple at the
# dict-and-deque surface, but skip the three User* wrappers entirely.
# mamba 0.3.60 supports every probed instance-method form below
# (constructor, item-set / item-get, contains, len, iteration via
# `dict(_)` / `list(_)`, the dict/list mutation API, and the str
# query API). The `.data` underlying-container attribute returns
# `None` on mamba (CPython returns the actual `dict`/`list`/`str`)
# — that gap is tracked separately and is NOT exercised here.
# Subclass override of dunder/method dispatch (e.g. a `MyDict`
# subclass with a custom `__setitem__`) also doesn't dispatch
# correctly on mamba — also tracked separately and NOT exercised
# here.
#
# Surface:
#   • UserDict() / UserDict(mapping) constructor;
#   • UserDict supports __setitem__, __getitem__, __contains__,
#     __len__, .get(), .pop(), .keys(), .values(), iteration via
#     dict() conversion;
#   • UserList() / UserList(iterable) constructor;
#   • UserList supports __setitem__, __getitem__, __contains__,
#     __len__, .append(), .extend(), .insert(), .pop(), .reverse(),
#     .count(), and slice access;
#   • UserString(s) constructor;
#   • UserString supports __getitem__, __contains__, __len__,
#     .upper(), .lower(), .startswith(), .endswith(), .replace(),
#     .split(), .strip(), .find(), .count().
from collections import UserDict, UserList, UserString
_ledger: list[int] = []

# UserDict — empty + mapping constructor + __setitem__
_ud = UserDict({'a': 1, 'b': 2})
_ud['c'] = 3
assert dict(_ud) == {'a': 1, 'b': 2, 'c': 3}; _ledger.append(1)

# UserDict — __contains__
assert 'a' in _ud; _ledger.append(1)
assert 'zzz' not in _ud; _ledger.append(1)

# UserDict — __len__
assert len(_ud) == 3; _ledger.append(1)

# UserDict — .get() hit + miss with default
assert _ud.get('a') == 1; _ledger.append(1)
assert _ud.get('missing', 99) == 99; _ledger.append(1)
assert _ud.get('missing') is None; _ledger.append(1)

# UserDict — .pop()
_popped = _ud.pop('a')
assert _popped == 1; _ledger.append(1)
assert 'a' not in _ud; _ledger.append(1)

# UserDict — .keys() / .values() — sorted to dodge dict ordering
# divergence on remaining-key probes
assert sorted(_ud.keys()) == ['b', 'c']; _ledger.append(1)
assert sorted(_ud.values()) == [2, 3]; _ledger.append(1)

# UserDict — empty + post-set
_ud2 = UserDict()
assert len(_ud2) == 0; _ledger.append(1)
_ud2['x'] = 1
assert dict(_ud2) == {'x': 1}; _ledger.append(1)

# UserList — iterable constructor + __getitem__
_ul = UserList([1, 2, 3])
assert _ul[0] == 1; _ledger.append(1)
assert _ul[2] == 3; _ledger.append(1)

# UserList — __len__
assert len(_ul) == 3; _ledger.append(1)

# UserList — __contains__
assert 2 in _ul; _ledger.append(1)
assert 99 not in _ul; _ledger.append(1)

# UserList — .append()
_ul.append(4)
assert list(_ul) == [1, 2, 3, 4]; _ledger.append(1)

# UserList — .extend()
_ul.extend([5, 6])
assert list(_ul) == [1, 2, 3, 4, 5, 6]; _ledger.append(1)

# UserList — .insert(0, x)
_ul.insert(0, 99)
assert list(_ul) == [99, 1, 2, 3, 4, 5, 6]; _ledger.append(1)

# UserList — .pop() default (last element)
_last = _ul.pop()
assert _last == 6; _ledger.append(1)
assert list(_ul) == [99, 1, 2, 3, 4, 5]; _ledger.append(1)

# UserList — slice access yields the same items
assert list(_ul[1:3]) == [1, 2]; _ledger.append(1)

# UserList — .reverse()
_ul.reverse()
assert list(_ul) == [5, 4, 3, 2, 1, 99]; _ledger.append(1)

# UserList — .count()
assert _ul.count(2) == 1; _ledger.append(1)
assert _ul.count(99) == 1; _ledger.append(1)
assert _ul.count(123) == 0; _ledger.append(1)

# UserList — empty constructor
_ul2 = UserList()
assert len(_ul2) == 0; _ledger.append(1)
_ul2.append("hi")
assert list(_ul2) == ["hi"]; _ledger.append(1)

# UserString — constructor + __getitem__
_us = UserString("hello")
assert _us[0] == "h"; _ledger.append(1)
assert _us[4] == "o"; _ledger.append(1)

# UserString — __len__
assert len(_us) == 5; _ledger.append(1)

# UserString — __contains__
assert "ell" in _us; _ledger.append(1)
assert "xyz" not in _us; _ledger.append(1)

# UserString — .upper() / .lower()
assert _us.upper() == "HELLO"; _ledger.append(1)
assert UserString("WORLD").lower() == "world"; _ledger.append(1)

# UserString — .startswith() / .endswith()
assert _us.startswith("he"); _ledger.append(1)
assert _us.endswith("lo"); _ledger.append(1)
assert not _us.startswith("lo"); _ledger.append(1)

# UserString — .replace()
assert _us.replace("l", "L") == "heLLo"; _ledger.append(1)

# UserString — .split() on a separator
assert UserString("a,b,c").split(",") == ["a", "b", "c"]; _ledger.append(1)

# UserString — .strip()
assert UserString("  hi  ").strip() == "hi"; _ledger.append(1)

# UserString — .find()
assert _us.find("l") == 2; _ledger.append(1)
assert _us.find("z") == -1; _ledger.append(1)

# UserString — .count()
assert _us.count("l") == 2; _ledger.append(1)
assert _us.count("z") == 0; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_collections_user_classes_ops {sum(_ledger)} asserts")
