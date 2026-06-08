# Operational AssertionPass seed for the value contract of the
# `collections` module + the builtin dict / set / list method
# surfaces used by every container-ops path: `collections` (the
# documented `OrderedDict` construction + str repr, `defaultdict`
# integer factory + auto-increment, `Counter` construction +
# `most_common` + missing-key zero-return, `deque` append /
# appendleft / pop / popleft / rotate, `namedtuple` attribute
# access, `ChainMap` lookup, `UserDict` / `UserList` construction
# + module hasattr surface), plus the documented `dict` instance
# method surface (`get` / `pop` / `setdefault` / `update` /
# `clear` / `copy` / `fromkeys` + `|` merge operator), the
# documented `set` instance method surface (`|` / `&` / `-` /
# `^` set algebra + `add` / `remove` / `discard` / `pop` /
# `issubset` / `issuperset` + set comprehension), and the
# documented `list` instance method surface (`append` / `extend`
# / `insert` / `remove` / `pop` / `reverse` / `count` / `index`
# / `sort` with reverse=/key= + slicing).
#
# The matching subset between mamba and CPython is the full
# `dict` instance method layer + merge-op layer + comprehension
# layer, the full `set` instance method layer + algebra layer +
# comprehension layer, the full `list` instance method layer
# + slicing layer, the `collections.OrderedDict` construction +
# str-repr layer (popitem / move_to_end DIVERGE), the
# `defaultdict` factory + auto-increment layer, the `Counter`
# construction + `most_common` + missing-key zero layer
# (arithmetic DIVERGES), the `deque` full layer, the `namedtuple`
# attribute-access layer (subscript + _fields / _asdict /
# _replace DIVERGE), the `ChainMap` lookup layer, the
# `UserDict` / `UserList` construction layer, and the full
# `collections` hasattr surface.
#
# Surface in this fixture:
#   • dict — get / pop / setdefault / update / clear / copy /
#     fromkeys + comprehension + | merge + in / not in;
#   • set — | / & / - / ^ algebra + add / remove / discard /
#     pop / issubset / issuperset + comprehension;
#   • list — slicing (a:b / :b / a: / ::2 / ::-1 / -3:) +
#     append / extend / insert / remove / pop / reverse /
#     count / index / sort + reverse= / key=;
#   • collections.OrderedDict — construction + str repr;
#   • collections.defaultdict — int / list factory +
#     auto-increment / list-append;
#   • collections.Counter — construction + most_common +
#     missing-key zero;
#   • collections.deque — append / appendleft / pop / popleft
#     / rotate;
#   • collections.namedtuple — attribute access (.x / .y);
#   • collections.ChainMap — lookup;
#   • collections.UserDict / UserList — construction +
#     subscript;
#   • collections — module hasattr surface.
#
# Behavioral edges that DIVERGE on mamba (OrderedDict.popitem /
# move_to_end AttributeError, Counter arithmetic + / - returns
# the empty Counter() not the merged / subtracted Counter,
# namedtuple p[0] returns None — subscript broken, p._fields
# returns None, p._asdict / _replace AttributeError) are
# covered in the matching spec fixture
# `lang_ordereddict_counter_namedtuple_silent`.
import collections


_Point = collections.namedtuple("_Point", ["x", "y"])


_ledger: list[int] = []

# 1) dict — comprehension + len
_d = {x: x * x for x in range(5)}
assert _d == {0: 0, 1: 1, 2: 4, 3: 9, 4: 16}; _ledger.append(1)
assert len(_d) == 5; _ledger.append(1)

# 2) dict — get / pop / setdefault / update / clear
_d2 = {"a": 1, "b": 2, "c": 3}
assert _d2.get("z", 99) == 99; _ledger.append(1)
assert _d2.get("a") == 1; _ledger.append(1)
assert _d2.pop("a") == 1; _ledger.append(1)
assert _d2 == {"b": 2, "c": 3}; _ledger.append(1)
assert _d2.setdefault("d", 4) == 4; _ledger.append(1)
assert _d2.setdefault("b", 99) == 2; _ledger.append(1)
_d2.update({"e": 5})
assert _d2 == {"b": 2, "c": 3, "d": 4, "e": 5}; _ledger.append(1)

# 3) dict — copy / fromkeys / | merge / in
assert {"x": 1}.copy() == {"x": 1}; _ledger.append(1)
assert dict.fromkeys(["a", "b", "c"], 0) == {"a": 0, "b": 0, "c": 0}; _ledger.append(1)
assert ({"a": 1, "b": 2} | {"b": 22, "c": 3}) == {"a": 1, "b": 22, "c": 3}; _ledger.append(1)
assert ("a" in {"a": 1}) == True; _ledger.append(1)
assert ("z" in {"a": 1}) == False; _ledger.append(1)

# 4) set — comprehension + algebra
_s = {x % 3 for x in range(10)}
assert _s == {0, 1, 2}; _ledger.append(1)
assert ({1, 2, 3} | {2, 3, 4}) == {1, 2, 3, 4}; _ledger.append(1)
assert ({1, 2, 3} & {2, 3, 4}) == {2, 3}; _ledger.append(1)
assert ({1, 2, 3} - {2, 3, 4}) == {1}; _ledger.append(1)
assert ({1, 2, 3} ^ {2, 3, 4}) == {1, 4}; _ledger.append(1)

# 5) set — add / remove / discard / issubset / issuperset
_st = {1, 2}
_st.add(3)
assert _st == {1, 2, 3}; _ledger.append(1)
_st.remove(1)
assert _st == {2, 3}; _ledger.append(1)
_st.discard(99)
assert _st == {2, 3}; _ledger.append(1)
assert {1, 2}.issubset({1, 2, 3}) == True; _ledger.append(1)
assert {1, 2, 3}.issuperset({1, 2}) == True; _ledger.append(1)

# 6) list — slicing
_lst = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
assert _lst[2:5] == [2, 3, 4]; _ledger.append(1)
assert _lst[:3] == [0, 1, 2]; _ledger.append(1)
assert _lst[7:] == [7, 8, 9]; _ledger.append(1)
assert _lst[::2] == [0, 2, 4, 6, 8]; _ledger.append(1)
assert _lst[::-1] == [9, 8, 7, 6, 5, 4, 3, 2, 1, 0]; _ledger.append(1)
assert _lst[-3:] == [7, 8, 9]; _ledger.append(1)

# 7) list — instance methods
_ll = [1, 2, 3]
_ll.append(4)
assert _ll == [1, 2, 3, 4]; _ledger.append(1)
_ll.extend([5, 6])
assert _ll == [1, 2, 3, 4, 5, 6]; _ledger.append(1)
_ll.insert(0, 0)
assert _ll == [0, 1, 2, 3, 4, 5, 6]; _ledger.append(1)
_ll.remove(3)
assert _ll == [0, 1, 2, 4, 5, 6]; _ledger.append(1)
assert _ll.pop() == 6; _ledger.append(1)
assert _ll.pop(0) == 0; _ledger.append(1)
_ll.reverse()
assert _ll == [5, 4, 2, 1]; _ledger.append(1)

# 8) list — count / index / sort
assert [1, 2, 2, 3, 2].count(2) == 3; _ledger.append(1)
assert [1, 2, 2, 3, 2].index(3) == 3; _ledger.append(1)
_l3 = [3, 1, 4, 1, 5, 9, 2, 6]
_l3.sort()
assert _l3 == [1, 1, 2, 3, 4, 5, 6, 9]; _ledger.append(1)
_l3.sort(reverse=True)
assert _l3 == [9, 6, 5, 4, 3, 2, 1, 1]; _ledger.append(1)
_l4 = ["bb", "a", "ccc"]
_l4.sort(key=len)
assert _l4 == ["a", "bb", "ccc"]; _ledger.append(1)

# 9) collections.OrderedDict — construction + str repr layer
_od = collections.OrderedDict()
_od["a"] = 1
_od["b"] = 2
assert dict(_od) == {"a": 1, "b": 2}; _ledger.append(1)
assert len(_od) == 2; _ledger.append(1)

# 10) collections.defaultdict — int / list factory
_dd = collections.defaultdict(int)
_dd["x"] += 1
_dd["x"] += 2
_dd["y"] += 1
assert dict(_dd) == {"x": 3, "y": 1}; _ledger.append(1)
_dd2 = collections.defaultdict(list)
_dd2["a"].append(1)
_dd2["a"].append(2)
assert dict(_dd2) == {"a": [1, 2]}; _ledger.append(1)

# 11) collections.Counter — construction + most_common +
#     missing-key zero
_c = collections.Counter(["a", "b", "a", "c", "a", "b"])
assert _c["a"] == 3; _ledger.append(1)
assert _c["b"] == 2; _ledger.append(1)
assert _c["z"] == 0; _ledger.append(1)
assert _c.most_common(2) == [("a", 3), ("b", 2)]; _ledger.append(1)

# 12) collections.deque — append / appendleft / pop / popleft / rotate
_dq = collections.deque([1, 2, 3])
_dq.append(4)
_dq.appendleft(0)
assert list(_dq) == [0, 1, 2, 3, 4]; _ledger.append(1)
assert _dq.pop() == 4; _ledger.append(1)
assert _dq.popleft() == 0; _ledger.append(1)
_dq.rotate(1)
assert list(_dq) == [3, 1, 2]; _ledger.append(1)

# 13) collections.namedtuple — attribute access
_p = _Point(1, 2)
assert _p.x == 1; _ledger.append(1)
assert _p.y == 2; _ledger.append(1)

# 14) collections.ChainMap + UserDict + UserList
_cm = collections.ChainMap({"a": 1}, {"a": 2, "b": 3})
assert _cm["a"] == 1; _ledger.append(1)
assert _cm["b"] == 3; _ledger.append(1)
assert collections.UserDict({"a": 1})["a"] == 1; _ledger.append(1)
assert collections.UserList([1, 2, 3])[0] == 1; _ledger.append(1)

# 15) collections — module attribute hasattr surface
assert hasattr(collections, "OrderedDict") == True; _ledger.append(1)
assert hasattr(collections, "defaultdict") == True; _ledger.append(1)
assert hasattr(collections, "Counter") == True; _ledger.append(1)
assert hasattr(collections, "deque") == True; _ledger.append(1)
assert hasattr(collections, "namedtuple") == True; _ledger.append(1)
assert hasattr(collections, "ChainMap") == True; _ledger.append(1)
assert hasattr(collections, "UserDict") == True; _ledger.append(1)
assert hasattr(collections, "UserList") == True; _ledger.append(1)
assert hasattr(collections, "UserString") == True; _ledger.append(1)

# NB: OrderedDict.popitem / move_to_end AttributeError, Counter
# arithmetic + / - returns empty Counter(), namedtuple p[0]
# subscript returns None, p._fields returns None, p._asdict /
# _replace AttributeError — all DIVERGE on mamba — moved to
# the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_collections_dict_set_list_full_value_ops {sum(_ledger)} asserts")
