use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/collections/abc_mapping_views_are_set_like.py`.
#[test]
fn test_gen_behavior_std_libs_collections_abc_mapping_views_are_set_like() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "abc_mapping_views_are_set_like"
# subject = "collections.abc.KeysView"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.abc.KeysView: mapping keys()/items()/values() are ABC instances and the set-like key/item views support set operations against plain sets, snapshotting at the time of the operation"""
import collections.abc as abc
from collections import UserDict

mymap = UserDict()
mymap["red"] = 5
assert isinstance(mymap.keys(), (abc.Set, abc.KeysView)), "keys is a Set/KeysView"
assert isinstance(mymap.items(), (abc.Set, abc.ItemsView)), "items is a Set/ItemsView"
assert isinstance(mymap.values(), (abc.Collection, abc.ValuesView)), "values is a ValuesView"

z = mymap.keys() | {"orange"}
assert isinstance(z, set), "keys | set -> set"
mymap["blue"] = 7  # added after the union; must not appear in z
assert sorted(z) == ["orange", "red"], f"keys union snapshot = {sorted(z)!r}"

iz = UserDict(red=5).items() | {("orange", 3)}
assert iz == {("orange", 3), ("red", 5)}, f"items union = {iz!r}"

print("abc_mapping_views_are_set_like OK")
"###);
    assert_output(&out, r###"abc_mapping_views_are_set_like OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/abc_mutablesequence_mixins.py`.
#[test]
fn test_gen_behavior_std_libs_collections_abc_mutablesequence_mixins() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "abc_mutablesequence_mixins"
# subject = "collections.abc.MutableSequence"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.abc.MutableSequence: a MutableSequence subclass implementing the abstract methods gets append/extend/reverse/pop/remove/+= mixed in over a backing list"""
from collections.abc import MutableSequence

class Seq(MutableSequence):
    def __init__(self):
        self._lst = []

    def __getitem__(self, i):
        return self._lst[i]

    def __setitem__(self, i, v):
        self._lst[i] = v

    def __delitem__(self, i):
        del self._lst[i]

    def __len__(self):
        return len(self._lst)

    def insert(self, i, v):
        self._lst.insert(i, v)


seq = Seq()
seq.append(0)
seq.extend((1, 2, 3, 4))
assert len(seq) == 5 and seq[3] == 3, "append/extend mixins"
seq.reverse()
assert seq[0] == 4, "reverse mixin"
seq.pop()
seq.remove(3)
assert list(seq) == [4, 2, 1], f"pop+remove = {list(seq)!r}"
seq += (10, 20)
assert seq[-1] == 20, "+= mixin"

print("abc_mutablesequence_mixins OK")
"###);
    assert_output(&out, r###"abc_mutablesequence_mixins OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/abc_mutableset_mixins_mutate.py`.
#[test]
fn test_gen_behavior_std_libs_collections_abc_mutableset_mixins_mutate() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "abc_mutableset_mixins_mutate"
# subject = "collections.abc.MutableSet"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.abc.MutableSet: a MutableSet subclass adding add/discard gets pop and the in-place operators ^=, |=, -= mixed in, mutating and returning the same object"""
from collections.abc import MutableSet

class MutableSetSubclass(MutableSet):
    def __init__(self, items=None):
        self._s = set(items or [])

    def __contains__(self, v):
        return v in self._s

    def __iter__(self):
        return iter(self._s)

    def __len__(self):
        return len(self._s)

    def add(self, v):
        self._s.add(v)

    def discard(self, v):
        self._s.discard(v)


m = MutableSetSubclass([5, 43, 2, 1])
popped = m.pop()
assert len(m) == 3 and popped not in m and popped in {5, 43, 2, 1}, "pop returns and removes a member"

m2 = MutableSetSubclass([1, 2, 3])
m2 ^= [3, 4]
assert set(m2) == {1, 2, 4}, "ixor"
m2 |= [10]
assert 10 in m2, "ior"
m2 -= [1]
assert 1 not in m2, "isub"

print("abc_mutableset_mixins_mutate OK")
"###);
    assert_output(&out, r###"abc_mutableset_mixins_mutate OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/abc_set_mixins_provide_algebra.py`.
#[test]
fn test_gen_behavior_std_libs_collections_abc_set_mixins_provide_algebra() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "abc_set_mixins_provide_algebra"
# subject = "collections.abc.Set"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.abc.Set: a Set subclass implementing __contains__/__iter__/__len__ gets &, |, -, ^, ordering, equality, and isdisjoint mixed in, returning the same Set subclass via _from_iterable"""
from collections.abc import Set

class MySet(Set):
    def __init__(self, items):
        self._items = set(items)

    def __contains__(self, x):
        return x in self._items

    def __iter__(self):
        return iter(self._items)

    def __len__(self):
        return len(self._items)


s1 = MySet((1, 2, 3))
s2 = MySet((3, 4, 5))
assert (s1 & s2) == MySet((3,)), "intersection"
assert set(s1 | s2) == {1, 2, 3, 4, 5}, "union"
assert set(s1 - s2) == {1, 2}, "difference"
assert set(s1 ^ s2) == {1, 2, 4, 5}, "symmetric difference"
assert isinstance(s1 | s2, MySet), "binary op returns the same Set subclass via _from_iterable"
assert MySet((1,)) < MySet((1, 2)) and MySet((1, 2)) > MySet((1,)), "proper subset/superset"
assert not (MySet((1, 2)) <= MySet((1,))), "not subset"
assert s1.isdisjoint(MySet((4, 5, 6))) and not s1.isdisjoint(MySet((1, 9))), "isdisjoint"

print("abc_set_mixins_provide_algebra OK")
"###);
    assert_output(&out, r###"abc_set_mixins_provide_algebra OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/abc_subclass_and_register.py`.
#[test]
fn test_gen_behavior_std_libs_collections_abc_subclass_and_register() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "abc_subclass_and_register"
# subject = "collections.abc.Hashable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.abc.Hashable: direct subclassing of the one-trick-pony ABCs makes issubclass true, and register() makes a structurally-unrelated class a virtual subclass"""
import collections.abc as abc

ponies = (abc.Hashable, abc.Iterable, abc.Iterator, abc.Reversible,
          abc.Sized, abc.Container, abc.Callable)

for base in ponies:
    class Derived(base):
        pass
    assert issubclass(Derived, base), f"subclass of {base.__name__}"
    assert not issubclass(int, Derived), "int is not a subclass of Derived"

for base in ponies:
    class Plain:
        __hash__ = None
    assert not issubclass(Plain, base), f"not yet {base.__name__}"
    base.register(Plain)
    assert issubclass(Plain, base), f"registered as a virtual subclass of {base.__name__}"

print("abc_subclass_and_register OK")
"###);
    assert_output(&out, r###"abc_subclass_and_register OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/chainmap_bool_and_iteration.py`.
#[test]
fn test_gen_behavior_std_libs_collections_chainmap_bool_and_iteration() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "chainmap_bool_and_iteration"
# subject = "collections.ChainMap"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.ChainMap: a ChainMap is truthy iff any underlying map is non-empty, and iteration visits each distinct key exactly once across all maps"""
from collections import ChainMap, OrderedDict

assert not ChainMap(), "empty is falsy"
assert not ChainMap({}, {}), "all-empty is falsy"
assert ChainMap({1: 2}, {}), "front non-empty is truthy"
assert ChainMap({}, {1: 2}), "back non-empty is truthy"
ordered = ChainMap(OrderedDict(a=1, b=2), OrderedDict(b=99, c=3))
assert sorted(ordered) == ["a", "b", "c"], "iteration visits each distinct key once"
assert ordered["b"] == 2, "front map wins for a duplicate key"

print("chainmap_bool_and_iteration OK")
"###);
    assert_output(&out, r###"chainmap_bool_and_iteration OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/chainmap_first_map_wins_lookup.py`.
#[test]
fn test_gen_behavior_std_libs_collections_chainmap_first_map_wins_lookup() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "chainmap_first_map_wins_lookup"
# subject = "collections.ChainMap"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.ChainMap: ChainMap looks up keys front-to-back so the first map shadows later ones; coercion to dict and .items() flatten with first-map-wins precedence"""
from collections import ChainMap

cm = ChainMap(dict(a=1, b=2), dict(b=20, c=30))
assert cm["a"] == 1 and cm["b"] == 2 and cm["c"] == 30, "front map shadows later maps"
assert dict(cm) == dict(a=1, b=2, c=30), f"flatten = {dict(cm)!r}"
assert dict(cm.items()) == dict(a=1, b=2, c=30), "items flatten with first-map-wins"

print("chainmap_first_map_wins_lookup OK")
"###);
    assert_output(&out, r###"chainmap_first_map_wins_lookup OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/chainmap_new_child_and_parents.py`.
#[test]
fn test_gen_behavior_std_libs_collections_chainmap_new_child_and_parents() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "chainmap_new_child_and_parents"
# subject = "collections.ChainMap"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.ChainMap: new_child pushes a fresh writable front map (writes/deletes touch only it, falling through to parents), and .parents drops the front map re-exposing inherited values"""
from collections import ChainMap

c = ChainMap()
c["a"] = 1
c["b"] = 2
d = c.new_child()
d["b"] = 20
d["c"] = 30
assert d.maps == [{"b": 20, "c": 30}, {"a": 1, "b": 2}], f"maps = {d.maps!r}"
assert d["a"] == 1 and d["b"] == 20 and len(d) == 3, "front map writes, parent fall-through"
del d["b"]
assert d.maps == [{"c": 30}, {"a": 1, "b": 2}], "del only touches the front map"
assert d["b"] == 2, "parent value re-exposed after del"
f = d.new_child()
f["b"] = 5
assert f["b"] == 5 and f.parents["b"] == 2, "parents drops the front map"

print("chainmap_new_child_and_parents OK")
"###);
    assert_output(&out, r###"chainmap_new_child_and_parents OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/chainmap_pop_only_front_map.py`.
#[test]
fn test_gen_behavior_std_libs_collections_chainmap_pop_only_front_map() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "chainmap_pop_only_front_map"
# subject = "collections.ChainMap"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.ChainMap: pop/popitem consider only the first map (pop returns the default when the key is gone), and popitem on a drained front map raises KeyError"""
from collections import ChainMap

cm = ChainMap(dict(a=1, b=2), dict(c=30))
assert cm.pop("a", 1001) == 1, "pop an existing front-map key"
assert cm.pop("a", 1002) == 1002, "pop default once the key is gone"
try:
    cm.pop("c")  # c lives only in the back map
    raise AssertionError("expected KeyError popping a back-map key")
except KeyError:
    pass
drain = ChainMap(dict(a=1, b=2), dict(b=20, c=30))
drain.popitem()
drain.popitem()
try:
    drain.popitem()
    raise AssertionError("expected KeyError when the front map is drained")
except KeyError:
    pass

print("chainmap_pop_only_front_map OK")
"###);
    assert_output(&out, r###"chainmap_pop_only_front_map OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/chainmap_union_operators.py`.
#[test]
fn test_gen_behavior_std_libs_collections_chainmap_union_operators() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "chainmap_union_operators"
# subject = "collections.ChainMap"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.ChainMap: PEP 584 | merges the other mapping into a copy of the first map preserving trailing maps; |= mutates in place; ChainMap | dict and dict | ChainMap fold correctly; result type follows the left operand"""
from collections import ChainMap

cm1 = ChainMap(dict(a=1, b=2), dict(c=3, d=4))
cm2 = ChainMap(dict(a=10, e=5), dict(b=20, d=4))

merged = cm1 | cm2
assert merged.maps == [cm1.maps[0] | dict(cm2), *cm1.maps[1:]], "| merges into a copy of the front map"

cm1_copy = ChainMap(dict(a=1, b=2), dict(c=3, d=4))
cm1_copy |= cm2
assert cm1_copy == merged, "|= matches |"

d = dict(a=10, c=30)
assert (cm2 | d).maps == [cm2.maps[0] | d, *cm2.maps[1:]], "ChainMap | dict folds into the front map"
assert (d | cm2).maps == [d | dict(cm2)], "dict | ChainMap yields a single-map ChainMap"

try:
    ChainMap() | [("c", 3)]
    raise AssertionError("expected TypeError for a non-mapping operand")
except TypeError:
    pass

class Sub(ChainMap):
    pass

assert type(ChainMap() | ChainMap()) is ChainMap, "base | base -> base"
assert type(ChainMap() | Sub()) is ChainMap, "base | sub -> base"
assert type(Sub() | ChainMap()) is Sub, "sub | base -> sub (result type follows the left)"

print("chainmap_union_operators OK")
"###);
    assert_output(&out, r###"chainmap_union_operators OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/counter_counts_from_iterable.py`.
#[test]
fn test_gen_behavior_std_libs_collections_counter_counts_from_iterable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "counter_counts_from_iterable"
# subject = "collections.Counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: Counter('abracadabra') tallies element frequencies; a present key returns its count and a missing key returns 0 without being inserted"""
from collections import Counter

c = Counter("abracadabra")
assert c["a"] == 5, f"a = {c['a']!r}"
assert c["b"] == 2, f"b = {c['b']!r}"
assert c["r"] == 2, f"r = {c['r']!r}"
assert c["z"] == 0, "missing key reads as 0"
assert "z" not in c, "reading a missing key does not insert it"

print("counter_counts_from_iterable OK")
"###);
    assert_output(&out, r###"counter_counts_from_iterable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/counter_is_dict_subclass.py`.
#[test]
fn test_gen_behavior_std_libs_collections_counter_is_dict_subclass() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "counter_is_dict_subclass"
# subject = "collections.Counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: Counter is a dict subclass: isinstance/issubclass of dict, == a plain dict of equal counts, canonical repr Counter({...}), and .get(missing, default) returns the default"""
from collections import Counter

c = Counter("abcaba")
assert isinstance(c, dict), "Counter is a dict instance"
assert issubclass(Counter, dict), "Counter subclasses dict"
assert c == dict(a=3, b=2, c=1), "equal to a plain dict of the same counts"
assert repr(c) == "Counter({'a': 3, 'b': 2, 'c': 1})", f"repr = {repr(c)!r}"
assert c.get("z", 10) == 10, ".get default for a missing key"
assert ("z" in c) is False, "missing key is not contained"

print("counter_is_dict_subclass OK")
"###);
    assert_output(&out, r###"counter_is_dict_subclass OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/counter_keeps_zero_and_negative_counts.py`.
#[test]
fn test_gen_behavior_std_libs_collections_counter_keeps_zero_and_negative_counts() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "counter_keeps_zero_and_negative_counts"
# subject = "collections.Counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: counts may reach zero or go negative and the keys persist until explicitly deleted (c['b'] -= 2 -> 0 still in c; c['e'] = -5 kept)"""
from collections import Counter

c = Counter("abcaba")  # a=3, b=2, c=1
c["a"] += 1   # 4
c["b"] -= 2   # 0
c["e"] = -5
assert c["b"] == 0 and "b" in c, "a zero count is kept until deleted"
assert c["e"] == -5, "a negative count is kept"

print("counter_keeps_zero_and_negative_counts OK")
"###);
    assert_output(&out, r###"counter_keeps_zero_and_negative_counts OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/counter_most_common_orders_by_count.py`.
#[test]
fn test_gen_behavior_std_libs_collections_counter_most_common_orders_by_count() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "counter_most_common_orders_by_count"
# subject = "collections.Counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: Counter.most_common(n) returns the n highest-count (element, count) pairs in descending-count order"""
from collections import Counter

assert Counter([1, 2, 2, 3, 3, 3]).most_common(1) == [(3, 3)], "single most common"
assert Counter("banana").most_common(2) == [("a", 3), ("n", 2)], "top two by count"
assert Counter("abracadabra").most_common(3) == [("a", 5), ("b", 2), ("r", 2)], "top three by count"

print("counter_most_common_orders_by_count OK")
"###);
    assert_output(&out, r###"counter_most_common_orders_by_count OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/counter_multiset_ops_match_set_ops.py`.
#[test]
fn test_gen_behavior_std_libs_collections_counter_multiset_ops_match_set_ops() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "counter_multiset_ops_match_set_ops"
# subject = "collections.Counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: on counts of 0/1 the multiset operators +, -, |, & agree with the corresponding set operations on the present elements"""
from collections import Counter

def to_set(counter):
    return set(counter.elements())

p = Counter(a=1, b=1, c=0)
q = Counter(b=1, c=1, d=1)
assert set((p + q).elements()) == to_set(p) | to_set(q), "add ~ union"
assert set((p - q).elements()) == to_set(p) - to_set(q), "sub ~ difference"
assert set((p | q).elements()) == to_set(p) | to_set(q), "or ~ union"
assert set((p & q).elements()) == to_set(p) & to_set(q), "and ~ intersection"

print("counter_multiset_ops_match_set_ops OK")
"###);
    assert_output(&out, r###"counter_multiset_ops_match_set_ops OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/counter_preserves_first_seen_order.py`.
#[test]
fn test_gen_behavior_std_libs_collections_counter_preserves_first_seen_order() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "counter_preserves_first_seen_order"
# subject = "collections.Counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: Counter preserves the first-occurrence insertion order of its keys: Counter('abracadabra').items() == [('a',5),('b',2),('r',2),('c',1),('d',1)]"""
from collections import Counter

assert list(Counter("abracadabra").items()) == [
    ("a", 5), ("b", 2), ("r", 2), ("c", 1), ("d", 1),
], "keys keep first-occurrence order"

print("counter_preserves_first_seen_order OK")
"###);
    assert_output(&out, r###"counter_preserves_first_seen_order OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/counter_update_accumulates.py`.
#[test]
fn test_gen_behavior_std_libs_collections_counter_update_accumulates() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "counter_update_accumulates"
# subject = "collections.Counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: Counter.update(mapping) adds the mapping's counts onto the existing tallies rather than replacing them"""
from collections import Counter

c = Counter(a=1)
c.update({"a": 2, "b": 1})
assert c["a"] == 3 and c["b"] == 1, f"update = {dict(c)!r}"

print("counter_update_accumulates OK")
"###);
    assert_output(&out, r###"counter_update_accumulates OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/defaultdict_factory_called_on_missing_only.py`.
#[test]
fn test_gen_behavior_std_libs_collections_defaultdict_factory_called_on_missing_only() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "defaultdict_factory_called_on_missing_only"
# subject = "collections.defaultdict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.defaultdict: the default_factory is invoked exactly once when a missing key is first read (observable via a logging factory) and not on subsequent reads"""
from collections import defaultdict

log = []
def factory():
    log.append("called")
    return 0

d = defaultdict(factory)
assert d["new_key"] == 0, "factory result used for a missing key"
assert log == ["called"], "factory invoked exactly once"
_ = d["new_key"]
assert log == ["called"], "factory not invoked again for a present key"

print("defaultdict_factory_called_on_missing_only OK")
"###);
    assert_output(&out, r###"defaultdict_factory_called_on_missing_only OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/defaultdict_int_factory_autocreates_zero.py`.
#[test]
fn test_gen_behavior_std_libs_collections_defaultdict_int_factory_autocreates_zero() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "defaultdict_int_factory_autocreates_zero"
# subject = "collections.defaultdict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.defaultdict: defaultdict(int) auto-creates a missing key with value 0 so += accumulates; membership, len, keys/values, get, and update behave like a plain dict"""
from collections import defaultdict

d = defaultdict(int)
d["a"] += 1
d["b"] += 2
d["a"] += 10
assert d["a"] == 11 and d["b"] == 2, f"accumulated = {dict(d)!r}"
assert d["missing"] == 0, "missing key auto-creates 0"
assert sorted(d.keys()) == ["a", "b", "missing"], "missing read inserts the key"
assert sorted(d.values()) == [0, 2, 11], "values"
assert ("a" in d) and ("nope" not in d), "membership"
assert d.get("a") == 11 and d.get("never", -1) == -1, "get with default"
d.update({"z": 100})
assert d["z"] == 100, "update sets a key"

print("defaultdict_int_factory_autocreates_zero OK")
"###);
    assert_output(&out, r###"defaultdict_int_factory_autocreates_zero OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/defaultdict_list_factory_groups_appends.py`.
#[test]
fn test_gen_behavior_std_libs_collections_defaultdict_list_factory_groups_appends() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "defaultdict_list_factory_groups_appends"
# subject = "collections.defaultdict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.defaultdict: defaultdict(list) auto-creates a fresh list per missing key so appends group values, and .default_factory is the list type itself"""
from collections import defaultdict

d = defaultdict(list)
d["x"].append(1)
d["x"].append(2)
d["y"].append(99)
assert d["x"] == [1, 2] and d["y"] == [99], f"grouped = {dict(d)!r}"
assert sorted(d.keys()) == ["x", "y"], "keys"
assert d.default_factory is list, "default_factory is the list type"

print("defaultdict_list_factory_groups_appends OK")
"###);
    assert_output(&out, r###"defaultdict_list_factory_groups_appends OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/deque_append_both_ends.py`.
#[test]
fn test_gen_behavior_std_libs_collections_deque_append_both_ends() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "deque_append_both_ends"
# subject = "collections.deque"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.deque: deque supports appendleft/append/popleft on both ends and iterates in order"""
from collections import deque

d = deque([1, 2, 3])
d.appendleft(0)
d.append(4)
assert list(d) == [0, 1, 2, 3, 4], f"after appends = {list(d)!r}"
d.popleft()
assert list(d) == [1, 2, 3, 4], f"after popleft = {list(d)!r}"

print("deque_append_both_ends OK")
"###);
    assert_output(&out, r###"deque_append_both_ends OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/deque_maxlen_drops_from_opposite_end.py`.
#[test]
fn test_gen_behavior_std_libs_collections_deque_maxlen_drops_from_opposite_end() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "deque_maxlen_drops_from_opposite_end"
# subject = "collections.deque"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.deque: a maxlen-bounded deque silently drops the oldest element on overflow rather than raising, keeping only the most recent maxlen items"""
from collections import deque

d = deque(maxlen=3)
for i in range(6):
    d.append(i)
assert list(d) == [3, 4, 5], f"bounded keeps last maxlen = {list(d)!r}"
seeded = deque([1, 2, 3, 4, 5], maxlen=3)
assert list(seeded) == [3, 4, 5], "construction also truncates to the most recent"

print("deque_maxlen_drops_from_opposite_end OK")
"###);
    assert_output(&out, r###"deque_maxlen_drops_from_opposite_end OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/deque_rotate_shifts_elements.py`.
#[test]
fn test_gen_behavior_std_libs_collections_deque_rotate_shifts_elements() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "deque_rotate_shifts_elements"
# subject = "collections.deque"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.deque: deque.rotate(n) shifts elements right by n (negative rotates left), wrapping around"""
from collections import deque

d = deque([1, 2, 3, 4, 5])
d.rotate(2)
assert list(d) == [4, 5, 1, 2, 3], f"rotate(2) = {list(d)!r}"
d.rotate(-1)
assert list(d) == [5, 1, 2, 3, 4], f"rotate(-1) = {list(d)!r}"

print("deque_rotate_shifts_elements OK")
"###);
    assert_output(&out, r###"deque_rotate_shifts_elements OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/namedtuple_class_introspection.py`.
#[test]
fn test_gen_behavior_std_libs_collections_namedtuple_class_introspection() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "namedtuple_class_introspection"
# subject = "collections.namedtuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.namedtuple: the generated class exposes __name__, empty __slots__, _fields, __match_args__, and reuses tuple.__getitem__"""
from collections import namedtuple

Point = namedtuple("Point", "x y")
assert Point.__name__ == "Point", "name"
assert Point.__slots__ == (), "empty slots"
assert Point._fields == ("x", "y"), "fields tuple"
assert Point.__match_args__ == ("x", "y"), "match args"
assert Point.__getitem__ == tuple.__getitem__, "reuses tuple.__getitem__"

print("namedtuple_class_introspection OK")
"###);
    assert_output(&out, r###"namedtuple_class_introspection OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/namedtuple_defaults_fill_rightmost.py`.
#[test]
fn test_gen_behavior_std_libs_collections_namedtuple_defaults_fill_rightmost() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "namedtuple_defaults_fill_rightmost"
# subject = "collections.namedtuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.namedtuple: the defaults= keyword fills the rightmost fields; _field_defaults reflects them; a single default leaves earlier fields required; defaults may be any iterable; defaults=None means none"""
from collections import namedtuple

Point = namedtuple("Point", "x y", defaults=(10, 20))
assert Point._field_defaults == {"x": 10, "y": 20}, "both fields defaulted"
assert Point(1, 2) == (1, 2) and Point(1) == (1, 20) and Point() == (10, 20), "defaults fill rightmost"

Partial = namedtuple("Partial", "x y", defaults=(20,))
assert Partial._field_defaults == {"y": 20}, "single default fills only the last field"
assert Partial(1) == (1, 20), "x still required"

FromIter = namedtuple("FromIter", "x y", defaults=iter([10, 20]))
assert FromIter() == (10, 20), "defaults may be any iterable"

NoneDef = namedtuple("NoneDef", "x y", defaults=None)
assert NoneDef._field_defaults == {}, "defaults=None means no defaults"

print("namedtuple_defaults_fill_rightmost OK")
"###);
    assert_output(&out, r###"namedtuple_defaults_fill_rightmost OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/namedtuple_field_spec_forms.py`.
#[test]
fn test_gen_behavior_std_libs_collections_namedtuple_field_spec_forms() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "namedtuple_field_spec_forms"
# subject = "collections.namedtuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.namedtuple: field names accept a space string, a comma string, or a sequence of names, all yielding the same _fields tuple"""
from collections import namedtuple

assert namedtuple("A", "p q")._fields == ("p", "q"), "space-separated string"
assert namedtuple("B", "p, q")._fields == ("p", "q"), "comma-separated string"
assert namedtuple("C", ("p", "q"))._fields == ("p", "q"), "tuple of names"
assert namedtuple("D", ["p", "q"])._fields == ("p", "q"), "list of names"

print("namedtuple_field_spec_forms OK")
"###);
    assert_output(&out, r###"namedtuple_field_spec_forms OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/namedtuple_make_from_iterable.py`.
#[test]
fn test_gen_behavior_std_libs_collections_namedtuple_make_from_iterable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "namedtuple_make_from_iterable"
# subject = "collections.namedtuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.namedtuple: _make builds an instance from an iterable and enforces the field count, raising TypeError on the wrong length"""
from collections import namedtuple

Point = namedtuple("Point", "x y")
assert Point._make([11, 22]) == (11, 22), "_make from a list"
for bad in ([11], [11, 22, 33]):
    try:
        Point._make(bad)
        raise AssertionError(f"expected TypeError for _make({bad!r})")
    except TypeError:
        pass

print("namedtuple_make_from_iterable OK")
"###);
    assert_output(&out, r###"namedtuple_make_from_iterable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/namedtuple_rename_replaces_invalid.py`.
#[test]
fn test_gen_behavior_std_libs_collections_namedtuple_rename_replaces_invalid() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "namedtuple_rename_replaces_invalid"
# subject = "collections.namedtuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.namedtuple: rename=True replaces each invalid/duplicate/keyword/leading-underscore field with _<index>"""
from collections import namedtuple

assert namedtuple("NT", ["abc", "def"], rename=True)._fields == ("abc", "_1"), "keyword field renamed"
assert namedtuple("NT", ["8efg", "9ghi"], rename=True)._fields == ("_0", "_1"), "leading-digit fields renamed"
assert namedtuple("NT", ["abc", "efg", "efg", "ghi"], rename=True)._fields == (
    "abc", "efg", "_2", "ghi",
), "duplicate field renamed"

print("namedtuple_rename_replaces_invalid OK")
"###);
    assert_output(&out, r###"namedtuple_rename_replaces_invalid OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/namedtuple_replace_and_asdict.py`.
#[test]
fn test_gen_behavior_std_libs_collections_namedtuple_replace_and_asdict() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "namedtuple_replace_and_asdict"
# subject = "collections.namedtuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.namedtuple: _replace returns a modified copy leaving the original unchanged, and _asdict returns a plain dict of the fields"""
from collections import namedtuple

Pt = namedtuple("Pt", ["x", "y"])
p = Pt(1, 2)
p2 = p._replace(y=99)
assert p2.x == 1 and p2.y == 99, f"_replace = {p2!r}"
assert p.y == 2, "original unchanged by _replace"
d = p._asdict()
assert isinstance(d, dict) and d == {"x": 1, "y": 2}, f"_asdict = {d!r}"

print("namedtuple_replace_and_asdict OK")
"###);
    assert_output(&out, r###"namedtuple_replace_and_asdict OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/ordereddict_move_to_end_reorders.py`.
#[test]
fn test_gen_behavior_std_libs_collections_ordereddict_move_to_end_reorders() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "ordereddict_move_to_end_reorders"
# subject = "collections.OrderedDict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.OrderedDict: OrderedDict.move_to_end(key) relocates the key to the end of the iteration order"""
from collections import OrderedDict

od = OrderedDict()
od["a"] = 1
od["b"] = 2
od["c"] = 3
assert list(od.keys()) == ["a", "b", "c"], "initial order"
od.move_to_end("a")
assert list(od.keys()) == ["b", "c", "a"], "a relocated to the end"

print("ordereddict_move_to_end_reorders OK")
"###);
    assert_output(&out, r###"ordereddict_move_to_end_reorders OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/ordereddict_preserves_insertion_order.py`.
#[test]
fn test_gen_behavior_std_libs_collections_ordereddict_preserves_insertion_order() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "ordereddict_preserves_insertion_order"
# subject = "collections.OrderedDict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.OrderedDict: OrderedDict iterates keys/values in insertion order regardless of key sort order"""
from collections import OrderedDict

od = OrderedDict()
od["z"] = 1
od["a"] = 2
od["m"] = 3
assert list(od.keys()) == ["z", "a", "m"], "keys in insertion order"
assert list(od.values()) == [1, 2, 3], "values in insertion order"
assert list(od.items()) == [("z", 1), ("a", 2), ("m", 3)], "items in insertion order"

print("ordereddict_preserves_insertion_order OK")
"###);
    assert_output(&out, r###"ordereddict_preserves_insertion_order OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/userdict_delegates_to_data.py`.
#[test]
fn test_gen_behavior_std_libs_collections_userdict_delegates_to_data() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "userdict_delegates_to_data"
# subject = "collections.UserDict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.UserDict: UserDict stores its payload in a public .data dict and delegates mapping ops; a __missing__ override customizes absent-key lookup while .get bypasses it"""
from collections import UserDict

ud = UserDict(a=1)
ud["b"] = 2
assert sorted(ud.items()) == [("a", 1), ("b", 2)], f"items = {sorted(ud.items())!r}"
assert ud.data == {"a": 1, "b": 2}, "payload lives in .data"
assert ud["a"] == 1 and "b" in ud and len(ud) == 2, "delegated mapping ops"

class WithMissing(UserDict):
    def __missing__(self, key):
        return 456

assert WithMissing()[123] == 456, "__missing__ customizes absent-key lookup"
assert WithMissing().get(123) is None, ".get bypasses __missing__"

print("userdict_delegates_to_data OK")
"###);
    assert_output(&out, r###"userdict_delegates_to_data OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/userlist_delegates_to_data.py`.
#[test]
fn test_gen_behavior_std_libs_collections_userlist_delegates_to_data() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "userlist_delegates_to_data"
# subject = "collections.UserList"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.UserList: UserList wraps a list in .data and forwards append/extend, iteration, indexing/len, and concatenation"""
from collections import UserList

ul = UserList([1, 2, 3])
ul.append(4)
ul.extend([5])
assert ul.data == [1, 2, 3, 4, 5], "payload lives in .data"
assert list(ul) == [1, 2, 3, 4, 5], "iteration"
assert ul[0] == 1 and ul[-1] == 5 and len(ul) == 5, "indexing/len"
assert (ul + [6])[-1] == 6, "concatenation"

print("userlist_delegates_to_data OK")
"###);
    assert_output(&out, r###"userlist_delegates_to_data OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections/userstring_delegates_to_data.py`.
#[test]
fn test_gen_behavior_std_libs_collections_userstring_delegates_to_data() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "userstring_delegates_to_data"
# subject = "collections.UserString"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.UserString: UserString wraps a str in .data and forwards string methods, indexing/len, and concatenation while staying a UserString"""
from collections import UserString

us = UserString("hello")
assert us.data == "hello", "payload lives in .data"
assert us.upper() == "HELLO", "string method delegation"
assert us[0] == "h" and len(us) == 5, "indexing/len"
assert str(us + " world") == "hello world", "concatenation"
assert isinstance(us, UserString), "stays a UserString"

print("userstring_delegates_to_data OK")
"###);
    assert_output(&out, r###"userstring_delegates_to_data OK
"###);
}
