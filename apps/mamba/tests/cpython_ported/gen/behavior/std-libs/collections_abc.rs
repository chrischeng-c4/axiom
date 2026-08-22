use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/collections_abc/abc_subclass_hierarchy.py`.
#[test]
fn test_gen_behavior_std_libs_collections_abc_abc_subclass_hierarchy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "abc_subclass_hierarchy"
# subject = "collections.abc.Iterator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Iterator: issubclass relations hold between the ABCs: Iterator<Iterable, MutableSequence<Sequence, MutableMapping<Mapping, MutableSet<Set, Sequence<Reversible"""
import collections.abc as abc

assert issubclass(abc.Iterator, abc.Iterable), "Iterator < Iterable"
assert issubclass(abc.MutableSequence, abc.Sequence), "MutableSequence < Sequence"
assert issubclass(abc.MutableMapping, abc.Mapping), "MutableMapping < Mapping"
assert issubclass(abc.MutableSet, abc.Set), "MutableSet < Set"
assert issubclass(abc.Sequence, abc.Reversible), "Sequence < Reversible"
print("abc_subclass_hierarchy OK")
"###);
    assert_output(&out, r###"abc_subclass_hierarchy OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections_abc/builtin_isinstance_relations.py`.
#[test]
fn test_gen_behavior_std_libs_collections_abc_builtin_isinstance_relations() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "builtin_isinstance_relations"
# subject = "collections.abc.Sequence"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Sequence: built-in list/dict/set/frozenset/tuple/str register as instances of the matching ABCs (list is MutableSequence, dict is MutableMapping, set is MutableSet, frozenset/tuple/str are Set/Sequence)"""
import collections.abc as abc

assert isinstance([], abc.Iterable), "list is Iterable"
assert isinstance([], abc.Sequence), "list is Sequence"
assert isinstance([], abc.MutableSequence), "list is MutableSequence"
assert isinstance({}, abc.Mapping), "dict is Mapping"
assert isinstance({}, abc.MutableMapping), "dict is MutableMapping"
assert isinstance(set(), abc.Set), "set is Set"
assert isinstance(set(), abc.MutableSet), "set is MutableSet"
assert isinstance(frozenset(), abc.Set), "frozenset is Set"
assert not isinstance(frozenset(), abc.MutableSet), "frozenset is not MutableSet"
assert isinstance((), abc.Sequence), "tuple is Sequence"
assert isinstance("", abc.Sequence), "str is Sequence"
print("builtin_isinstance_relations OK")
"###);
    assert_output(&out, r###"builtin_isinstance_relations OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections_abc/custom_call_is_callable.py`.
#[test]
fn test_gen_behavior_std_libs_collections_abc_custom_call_is_callable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "custom_call_is_callable"
# subject = "collections.abc.Callable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Callable: a class defining __call__ is recognized as Callable and is invokable"""
import collections.abc as abc


class MyCallable:
    def __call__(self, x):
        return x * 2


assert isinstance(MyCallable(), abc.Callable), "custom __call__ is Callable"
assert MyCallable()(5) == 10, "callable result"
print("custom_call_is_callable OK")
"###);
    assert_output(&out, r###"custom_call_is_callable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections_abc/custom_contains_is_container.py`.
#[test]
fn test_gen_behavior_std_libs_collections_abc_custom_contains_is_container() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "custom_contains_is_container"
# subject = "collections.abc.Container"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Container: a class defining __contains__ is a Container and drives the `in` operator"""
import collections.abc as abc


class MyContainer:
    def __contains__(self, item):
        return item in {1, 2, 3}


assert isinstance(MyContainer(), abc.Container), "custom __contains__ is Container"
assert 1 in MyContainer(), "container contains 1"
assert 9 not in MyContainer(), "container lacks 9"
print("custom_contains_is_container OK")
"###);
    assert_output(&out, r###"custom_contains_is_container OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections_abc/custom_iter_is_iterable.py`.
#[test]
fn test_gen_behavior_std_libs_collections_abc_custom_iter_is_iterable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "custom_iter_is_iterable"
# subject = "collections.abc.Iterable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Iterable: a class defining __iter__ is recognized as Iterable via structural subclass hook, while a plain object is not"""
import collections.abc as abc


class MyIterable:
    def __iter__(self):
        return iter([1, 2, 3])


assert isinstance(MyIterable(), abc.Iterable), "custom __iter__ is Iterable"
assert not isinstance(object(), abc.Iterable), "plain object not Iterable"
print("custom_iter_is_iterable OK")
"###);
    assert_output(&out, r###"custom_iter_is_iterable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections_abc/custom_iterator_protocol.py`.
#[test]
fn test_gen_behavior_std_libs_collections_abc_custom_iterator_protocol() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "custom_iterator_protocol"
# subject = "collections.abc.Iterator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Iterator: a class defining __iter__ and __next__ is an Iterator and iterates to exhaustion via list()"""
import collections.abc as abc


class MyIterator:
    def __init__(self):
        self._data = [10, 20, 30]
        self._idx = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self._idx >= len(self._data):
            raise StopIteration
        v = self._data[self._idx]
        self._idx += 1
        return v


it = MyIterator()
assert isinstance(it, abc.Iterator), "custom Iterator"
assert isinstance(it, abc.Iterable), "Iterator is Iterable"
assert list(it) == [10, 20, 30], "Iterator iteration"
print("custom_iterator_protocol OK")
"###);
    assert_output(&out, r###"custom_iterator_protocol OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections_abc/custom_len_is_sized.py`.
#[test]
fn test_gen_behavior_std_libs_collections_abc_custom_len_is_sized() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "custom_len_is_sized"
# subject = "collections.abc.Sized"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Sized: a class defining __len__ is recognized as Sized via the structural subclass hook"""
import collections.abc as abc


class MySized:
    def __len__(self):
        return 5


assert isinstance(MySized(), abc.Sized), "custom __len__ is Sized"
assert len(MySized()) == 5, "len delegates to __len__"
print("custom_len_is_sized OK")
"###);
    assert_output(&out, r###"custom_len_is_sized OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections_abc/generator_is_iterator_iterable.py`.
#[test]
fn test_gen_behavior_std_libs_collections_abc_generator_is_iterator_iterable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "generator_is_iterator_iterable"
# subject = "collections.abc.Generator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Generator: a generator object is an instance of Generator, Iterator, and Iterable"""
import collections.abc as abc


def gen():
    yield 1
    yield 2


g = gen()
assert isinstance(g, abc.Generator), "generator is Generator"
assert isinstance(g, abc.Iterator), "generator is Iterator"
assert isinstance(g, abc.Iterable), "generator is Iterable"
print("generator_is_iterator_iterable OK")
"###);
    assert_output(&out, r###"generator_is_iterator_iterable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections_abc/hashable_sized_container_builtins.py`.
#[test]
fn test_gen_behavior_std_libs_collections_abc_hashable_sized_container_builtins() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "hashable_sized_container_builtins"
# subject = "collections.abc.Hashable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Hashable: int/str/tuple are Hashable while list/dict/set are not; list/dict are Sized and Container"""
import collections.abc as abc

# Immutable built-ins are Hashable.
assert isinstance(42, abc.Hashable), "int is Hashable"
assert isinstance("hello", abc.Hashable), "str is Hashable"
assert isinstance((), abc.Hashable), "tuple is Hashable"
# Mutable built-ins are not Hashable.
assert not isinstance([], abc.Hashable), "list not Hashable"
assert not isinstance({}, abc.Hashable), "dict not Hashable"
assert not isinstance(set(), abc.Hashable), "set not Hashable"
# Sized / Container membership.
assert isinstance([], abc.Sized), "list is Sized"
assert isinstance({}, abc.Sized), "dict is Sized"
assert isinstance([], abc.Container), "list is Container"
assert isinstance({}, abc.Container), "dict is Container"
print("hashable_sized_container_builtins OK")
"###);
    assert_output(&out, r###"hashable_sized_container_builtins OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections_abc/mapping_mixin_methods.py`.
#[test]
fn test_gen_behavior_std_libs_collections_abc_mapping_mixin_methods() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "mapping_mixin_methods"
# subject = "collections.abc.Mapping"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Mapping: nominal subclasses inherit read-only mapping mixins"""
import collections.abc as abc


class MyMapping(abc.Mapping):
    def __init__(self, pairs):
        self.data = dict(pairs)

    def __getitem__(self, key):
        return self.data[key]

    def __iter__(self):
        return iter(self.data)

    def __len__(self):
        return len(self.data)


mapping = MyMapping([("red", 5), ("blue", 7)])

assert "red" in mapping, "__contains__ finds present keys"
assert "green" not in mapping, "__contains__ rejects missing keys"
assert mapping.get("red") == 5, "get returns existing value"
assert mapping.get("green") is None, "get defaults to None"
assert mapping.get("green", 11) == 11, "get returns explicit default"
assert list(mapping.keys()) == ["red", "blue"], "keys view iterates keys"
assert list(mapping.values()) == [5, 7], "values view iterates values"
assert list(mapping.items()) == [("red", 5), ("blue", 7)], "items view iterates pairs"
assert dict(mapping.items()) == {"red": 5, "blue": 7}, "items can materialize a dict"
assert mapping == {"red": 5, "blue": 7}, "mapping equality compares key/value pairs"
assert mapping != {"red": 5}, "mapping inequality detects value differences"

try:
    mapping["green"]
    raise AssertionError("missing mapping key should raise KeyError")
except KeyError:
    pass

print("mapping_mixin_methods OK")
"###);
    assert_output(&out, r###"mapping_mixin_methods OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections_abc/mutable_mapping_mixin_methods.py`.
#[test]
fn test_gen_behavior_std_libs_collections_abc_mutable_mapping_mixin_methods() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "mutable_mapping_mixin_methods"
# subject = "collections.abc.MutableMapping"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.MutableMapping: nominal subclasses inherit mutating mapping mixins"""
import collections.abc as abc


class MyMutableMapping(abc.MutableMapping):
    def __init__(self, pairs=()):
        self.data = dict(pairs)

    def __getitem__(self, key):
        return self.data[key]

    def __setitem__(self, key, value):
        self.data[key] = value

    def __delitem__(self, key):
        del self.data[key]

    def __iter__(self):
        return iter(self.data)

    def __len__(self):
        return len(self.data)


mapping = MyMutableMapping([("red", 5)])

assert mapping.setdefault("red", 99) == 5, "setdefault returns existing value"
assert mapping.setdefault("blue", 7) == 7, "setdefault inserts missing default"
assert mapping.data == {"red": 5, "blue": 7}, "setdefault mutates missing keys"

mapping.update({"green": 9})
mapping.update([("orange", 3)])
assert mapping.data == {"red": 5, "blue": 7, "green": 9, "orange": 3}, "update accepts mappings and pair iterables"

assert mapping.pop("green") == 9, "pop removes existing key"
assert mapping.pop("missing", 42) == 42, "pop returns explicit default"
try:
    mapping.pop("missing")
    raise AssertionError("pop without default should reject missing keys")
except KeyError:
    pass

removed_key, removed_value = mapping.popitem()
assert removed_key not in mapping, "popitem removes the returned key"
assert removed_value in (3, 5, 7), "popitem returns a stored value"

mapping.clear()
assert mapping.data == {}, "clear removes all items"

mapping.update(red=5, blue=7)
assert mapping.data == {"red": 5, "blue": 7}, "update accepts keyword items"
assert list(mapping.keys()) == ["red", "blue"], "MutableMapping inherits Mapping keys"
assert list(mapping.items()) == [("red", 5), ("blue", 7)], "MutableMapping inherits Mapping items"
assert "red" in mapping, "MutableMapping inherits Mapping __contains__"

native = {"a": 1}
native.update({"b": 2})
assert native.setdefault("b", 9) == 2, "native dict setdefault behavior remains intact"
assert native.pop("a") == 1, "native dict pop behavior remains intact"

print("mutable_mapping_mixin_methods OK")
"###);
    assert_output(&out, r###"mutable_mapping_mixin_methods OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections_abc/mutable_sequence_mixin_completeness.py`.
#[test]
fn test_gen_behavior_std_libs_collections_abc_mutable_sequence_mixin_completeness() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "mutable_sequence_mixin_completeness"
# subject = "collections.abc.MutableSequence"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.MutableSequence: nominal subclasses inherit the full mutable sequence mixin surface"""
import collections.abc as abc


class MyMutableSeq(abc.MutableSequence):
    def __init__(self, values=()):
        self.items = list(values)

    def __getitem__(self, index):
        return self.items[index]

    def __setitem__(self, index, value):
        self.items[index] = value

    def __delitem__(self, index):
        del self.items[index]

    def __len__(self):
        return len(self.items)

    def insert(self, index, value):
        self.items.insert(index, value)


seq = MyMutableSeq([1, 2])

seq.extend([3, 4])
assert seq.items == [1, 2, 3, 4], "extend appends every item through insert"
assert list(seq) == [1, 2, 3, 4], "__iter__ yields indexed values"
assert 3 in seq, "__contains__ finds present values"
assert 9 not in seq, "__contains__ rejects missing values"
assert seq.index(3) == 2, "index returns the first matching position"

assert seq.pop() == 4, "pop defaults to the last item"
assert seq.items == [1, 2, 3], "pop removes the default item"
assert seq.pop(0) == 1, "pop accepts an explicit index"
assert seq.items == [2, 3], "indexed pop removes from that position"

seq.remove(2)
assert seq.items == [3], "remove deletes the first matching value"

same = seq
seq += (5, 6)
assert seq is same, "__iadd__ returns self"
assert seq.items == [3, 5, 6], "__iadd__ extends from an iterable"

seq.reverse()
assert seq.items == [6, 5, 3], "reverse remains available with the complete mixin set"
seq.append(7)
assert seq.items == [6, 5, 3, 7], "append remains available with the complete mixin set"

try:
    seq.index(99)
    raise AssertionError("index should reject missing values")
except ValueError:
    pass


class ReadOnlySeq(abc.Sequence):
    def __init__(self, values):
        self.values = list(values)

    def __getitem__(self, index):
        return self.values[index]

    def __len__(self):
        return len(self.values)


readonly = ReadOnlySeq([1, 2, 3])
assert list(readonly) == [1, 2, 3], "unrelated Sequence protocol still iterates"
assert not hasattr(readonly, "append"), "Sequence subclass does not gain MutableSequence append"
assert not hasattr(readonly, "remove"), "Sequence subclass does not gain MutableSequence remove"

native = [1]
native.extend([2])
native += [3]
assert native.pop() == 3, "native list pop behavior remains intact"
native.remove(1)
assert native == [2], "native list remove behavior remains intact"

print("mutable_sequence_mixin_completeness OK")
"###);
    assert_output(&out, r###"mutable_sequence_mixin_completeness OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections_abc/mutable_sequence_mixin_methods.py`.
#[test]
fn test_gen_behavior_std_libs_collections_abc_mutable_sequence_mixin_methods() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "mutable_sequence_mixin_methods"
# subject = "collections.abc.MutableSequence"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.MutableSequence: subclassing MutableSequence with the five abstract methods yields working mixin methods append() and reverse()"""
import collections.abc as abc


class MyMutableSeq(abc.MutableSequence):
    def __init__(self):
        self._data = []

    def __getitem__(self, i):
        return self._data[i]

    def __setitem__(self, i, v):
        self._data[i] = v

    def __delitem__(self, i):
        del self._data[i]

    def __len__(self):
        return len(self._data)

    def insert(self, i, v):
        self._data.insert(i, v)


ms = MyMutableSeq()
# append() is a mixin method provided by MutableSequence.
ms.append(10)
ms.append(20)
assert len(ms) == 2, f"MutableSeq len = {len(ms)!r}"
assert ms[0] == 10, f"MutableSeq[0] = {ms[0]!r}"
# reverse() is a mixin method provided by MutableSequence.
ms.reverse()
assert ms[0] == 20, f"after reverse[0] = {ms[0]!r}"
print("mutable_sequence_mixin_methods OK")
"###);
    assert_output(&out, r###"mutable_sequence_mixin_methods OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections_abc/register_virtual_mapping.py`.
#[test]
fn test_gen_behavior_std_libs_collections_abc_register_virtual_mapping() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "register_virtual_mapping"
# subject = "collections.abc.Mapping"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Mapping: register() marks virtual subclasses without installing mapping mixins"""
import collections.abc as abc


class CustomMapping:
    def __getitem__(self, key):
        return key

    def __len__(self):
        return 0

    def __iter__(self):
        return iter([])


# Before registration: not a virtual subclass and no Mapping mixins are installed.
assert not isinstance(CustomMapping(), abc.Mapping), "unregistered not a Mapping"
assert not hasattr(CustomMapping, "get"), "unregistered class has no get mixin"
assert not hasattr(CustomMapping(), "items"), "unregistered instance has no items mixin"

abc.Mapping.register(CustomMapping)

assert isinstance(CustomMapping(), abc.Mapping), "registered Mapping"
assert issubclass(CustomMapping, abc.Mapping), "registered Mapping subclass"
assert not hasattr(CustomMapping, "get"), "virtual registration does not install class get"
assert not hasattr(CustomMapping(), "items"), "virtual registration does not install instance items"

try:
    CustomMapping().get("x")
    raise AssertionError("virtual registration must not provide get")
except AttributeError:
    pass

print("register_virtual_mapping OK")
"###);
    assert_output(&out, r###"register_virtual_mapping OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections_abc/register_virtual_mutablesequence_no_mixins.py`.
#[test]
fn test_gen_behavior_std_libs_collections_abc_register_virtual_mutablesequence_no_mixins() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "register_virtual_mutablesequence_no_mixins"
# subject = "collections.abc.MutableSequence.register"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.MutableSequence.register: virtual registration does not install mixin methods"""
import collections.abc as abc


class VirtualSeq:
    def __len__(self):
        return 0

    def __getitem__(self, index):
        raise IndexError(index)

    def __setitem__(self, index, value):
        raise IndexError(index)

    def __delitem__(self, index):
        raise IndexError(index)

    def insert(self, index, value):
        raise AssertionError("virtual sequence insert should not be called")


assert not hasattr(VirtualSeq, "append"), "unregistered class has no append mixin"
assert not hasattr(VirtualSeq(), "append"), "unregistered instance has no append mixin"

abc.MutableSequence.register(VirtualSeq)

assert issubclass(VirtualSeq, abc.MutableSequence), "registered class is a virtual subclass"
assert isinstance(VirtualSeq(), abc.MutableSequence), "registered instance is a virtual instance"
assert not hasattr(VirtualSeq, "append"), "virtual registration does not install class append"
assert not hasattr(VirtualSeq(), "append"), "virtual registration does not install instance append"

try:
    VirtualSeq().append("x")
    raise AssertionError("virtual registration must not provide append")
except AttributeError:
    pass


class NominalSeq(abc.MutableSequence):
    def __init__(self):
        self.items = []

    def __len__(self):
        return len(self.items)

    def __getitem__(self, index):
        return self.items[index]

    def __setitem__(self, index, value):
        self.items[index] = value

    def __delitem__(self, index):
        del self.items[index]

    def insert(self, index, value):
        self.items.insert(index, value)


nominal = NominalSeq()
assert hasattr(NominalSeq, "append"), "nominal subclass inherits append mixin"
assert hasattr(nominal, "append"), "nominal instance exposes append mixin"
nominal.append("x")
assert nominal.items == ["x"], "nominal append mixin remains available"

print("register_virtual_mutablesequence_no_mixins OK")
"###);
    assert_output(&out, r###"register_virtual_mutablesequence_no_mixins OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/collections_abc/reversible_builtins.py`.
#[test]
fn test_gen_behavior_std_libs_collections_abc_reversible_builtins() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "reversible_builtins"
# subject = "collections.abc.Reversible"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Reversible: tuple/str/list are Reversible and (since 3.8) dict is Reversible too"""
import collections.abc as abc

assert isinstance((), abc.Reversible), "tuple is Reversible"
assert isinstance("", abc.Reversible), "str is Reversible"
assert isinstance([], abc.Reversible), "list is Reversible"
assert isinstance({}, abc.Reversible), "dict is Reversible since 3.8"
print("reversible_builtins OK")
"###);
    assert_output(&out, r###"reversible_builtins OK
"###);
}
