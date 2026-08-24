use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/copy/copy_atomic_returns_identity.py`.
#[test]
fn test_gen_behavior_std_libs_copy_copy_atomic_returns_identity() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "copy_atomic_returns_identity"
# subject = "copy.copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.copy: copy.copy of immutable atoms (None, ints, floats, str, bytes, range, slice, frozenset, type, function code, empty/immutable tuples) returns the very same object"""
import copy


def f():
    return None


class NewStyle:
    pass


# For these immutable, non-recursive values copy.copy returns the original
# object by identity — no new object is built.
atoms = [
    None, ..., NotImplemented, 42, 2 ** 100, 3.14, True, False, 1j,
    "hello", "héllo", b"world", bytes(range(8)),
    f.__code__, range(10), slice(1, 10, 2),
    NewStyle, max, property(), frozenset({1, 2, 3}), frozenset(),
    (), (1, 2, 3),
]
for x in atoms:
    assert copy.copy(x) is x, f"copy.copy should return identity for {x!r}"

print("copy_atomic_returns_identity OK")
"###);
    assert_output(&out, r###"copy_atomic_returns_identity OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/copy_mutable_containers_new_but_equal.py`.
#[test]
fn test_gen_behavior_std_libs_copy_copy_mutable_containers_new_but_equal() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "copy_mutable_containers_new_but_equal"
# subject = "copy.copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.copy: shallow copy of bytearray/set/list/dict (including empty ones) yields a distinct object that compares equal, with mutable members still shared"""
import copy

# bytearray: shallow copy is a new, equal buffer.
ba = bytearray(b"abc")
ba_c = copy.copy(ba)
assert ba_c == ba and ba_c is not ba, "bytearray copy new-but-equal"
empty_ba = copy.copy(bytearray())
assert empty_ba == bytearray() and empty_ba is not bytearray(), "empty bytearray copy"

# set: shallow copy is a new, equal set.
s = {1, 2, 3}
assert copy.copy(s) == s and copy.copy(s) is not s, "set copy new-but-equal"
assert copy.copy(set()) == set(), "empty set copy"

# list: shallow copy new outer, equal contents.
ls = [1, 2, 3]
assert copy.copy(ls) == ls and copy.copy(ls) is not ls, "list copy new-but-equal"
assert copy.copy([]) == [], "empty list copy"

# dict: shallow copy new outer, equal contents.
d = {"foo": 1, "bar": 2}
assert copy.copy(d) == d and copy.copy(d) is not d, "dict copy new-but-equal"
assert copy.copy({}) == {}, "empty dict copy"

# Shallow copy shares mutable members; deepcopy does not.
nested = {"x": [1, 2], "y": [3, 4]}
assert copy.copy(nested)["x"] is nested["x"], "shallow dict shares inner list"
deep = copy.deepcopy(nested)
assert deep["x"] is not nested["x"], "deep dict copies inner list"
deep["x"].append(99)
assert nested["x"] == [1, 2], "deep copy is independent of the original"

print("copy_mutable_containers_new_but_equal OK")
"###);
    assert_output(&out, r###"copy_mutable_containers_new_but_equal OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/deepcopy_atomic_returns_identity.py`.
#[test]
fn test_gen_behavior_std_libs_copy_deepcopy_atomic_returns_identity() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "deepcopy_atomic_returns_identity"
# subject = "copy.deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: deepcopy of immutable atoms and tuples-of-immutables returns the same object, but a tuple holding a mutable member is rebuilt with an independent inner copy"""
import copy


def f():
    return None


class NewStyle:
    pass


# deepcopy returns identity for immutable atoms, plus tuples whose members are
# themselves all immutable (no deep work needed).
atoms = [
    None, ..., NotImplemented, 42, 2 ** 100, 3.14, True, False, 1j,
    "hello", b"world", f.__code__, NewStyle, range(10), max, property(),
    (), ((1, 2), 3),
]
for x in atoms:
    assert copy.deepcopy(x) is x, f"deepcopy should return identity for {x!r}"

# A tuple holding a mutable member is NOT atomic: deepcopy rebuilds it so the
# inner mutable is independent.
nested = ([1, 2], 3)
deep = copy.deepcopy(nested)
assert deep == nested and deep is not nested, "deepcopy of nested tuple is new but equal"
assert deep[0] is not nested[0], "deepcopy copies the inner mutable list"

print("deepcopy_atomic_returns_identity OK")
"###);
    assert_output(&out, r###"deepcopy_atomic_returns_identity OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/deepcopy_bound_method_rebinds.py`.
#[test]
fn test_gen_behavior_std_libs_copy_deepcopy_bound_method_rebinds() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "deepcopy_bound_method_rebinds"
# subject = "copy.deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: deepcopying an object that stores one of its own bound methods rebinds the method's __self__ to the copy, not the original"""
import copy


class Bound:
    def m(self):
        return self


b = Bound()
b.b = b.m  # store a bound method of self as an attribute
bd = copy.deepcopy(b)
assert bd.b.__self__ is bd, "deepcopy rebinds the bound method __self__ to the copy"
assert bd.m == bd.b, "the rebound method equals a freshly-bound method on the copy"

print("deepcopy_bound_method_rebinds OK")
"###);
    assert_output(&out, r###"deepcopy_bound_method_rebinds OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/deepcopy_instance_recursive.py`.
#[test]
fn test_gen_behavior_std_libs_copy_deepcopy_instance_recursive() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "deepcopy_instance_recursive"
# subject = "copy.deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: deepcopy of a linked-node instance copies the chain into fresh, independent instances"""
import copy


class Node:
    def __init__(self, val, child=None):
        self.val = val
        self.child = child


root = Node(1, Node(2))
deep = copy.deepcopy(root)
assert deep is not root, "deepcopy of the root is a new instance"
assert deep.val == 1, f"root value preserved = {deep.val!r}"
assert deep.child.val == 2, f"child value preserved = {deep.child.val!r}"
assert deep.child is not root.child, "the child node is a fresh instance too"

print("deepcopy_instance_recursive OK")
"###);
    assert_output(&out, r###"deepcopy_instance_recursive OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/deepcopy_memo_shares_repeated_ref.py`.
#[test]
fn test_gen_behavior_std_libs_copy_deepcopy_memo_shares_repeated_ref() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "deepcopy_memo_shares_repeated_ref"
# subject = "copy.deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: the memo makes two references to the same object inside one structure stay shared (and equal a single new object) after deepcopy"""
import copy

shared = [99]
container = [shared, shared]  # both elements point to the same list
deep = copy.deepcopy(container)
assert deep[0] is deep[1], "memo keeps the repeated reference shared in the copy"
assert deep[0] is not shared, "the shared element is a fresh copy, not the original"

print("deepcopy_memo_shares_repeated_ref OK")
"###);
    assert_output(&out, r###"deepcopy_memo_shares_repeated_ref OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/deepcopy_nested_independent.py`.
#[test]
fn test_gen_behavior_std_libs_copy_deepcopy_nested_independent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "deepcopy_nested_independent"
# subject = "copy.deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: deepcopy of a doubly-nested dict makes every level an independent object so mutating a deep value leaves the original untouched"""
import copy

original = {"a": [1, [2, 3]], "b": {"c": [4, 5]}}
deep = copy.deepcopy(original)
assert deep == original and deep is not original, "deep outer is new"
assert deep["a"] is not original["a"], "deep copies the inner list"
assert deep["b"]["c"] is not original["b"]["c"], "deep copies the doubly-nested list"

deep["a"].append(99)
assert original["a"] == [1, [2, 3]], "deep copy is independent of the original"

print("deepcopy_nested_independent OK")
"###);
    assert_output(&out, r###"deepcopy_nested_independent OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/deepcopy_reflexive_instance.py`.
#[test]
fn test_gen_behavior_std_libs_copy_deepcopy_reflexive_instance() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "deepcopy_reflexive_instance"
# subject = "copy.deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: deepcopy of an instance that stores a reference to itself rebuilds the self-reference through the memo"""
import copy


class Cyclic:
    pass


c = Cyclic()
c.foo = c  # self-referential instance attribute
cd = copy.deepcopy(c)
assert cd is not c, "deepcopy is a new instance"
assert cd.foo is cd, "the self-reference is rebuilt to point at the copy, not the original"

print("deepcopy_reflexive_instance OK")
"###);
    assert_output(&out, r###"deepcopy_reflexive_instance OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/deepcopy_reflexive_list_cycle.py`.
#[test]
fn test_gen_behavior_std_libs_copy_deepcopy_reflexive_list_cycle() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "deepcopy_reflexive_list_cycle"
# subject = "copy.deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: deepcopy of a self-referential list rebuilds the cycle: the copy points to itself, not to the original"""
import copy

cyclic = [1, 2]
cyclic.append(cyclic)  # self-referential
copied = copy.deepcopy(cyclic)
assert copied is not cyclic, "deepcopy is a new list"
assert copied[2] is copied, "cycle is preserved: the copy points to itself"
assert copied[2] is not cyclic, "the cycle does not leak back to the original"

print("deepcopy_reflexive_list_cycle OK")
"###);
    assert_output(&out, r###"deepcopy_reflexive_list_cycle OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/deepcopy_tuple_with_mutable_rebuilt.py`.
#[test]
fn test_gen_behavior_std_libs_copy_deepcopy_tuple_with_mutable_rebuilt() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "deepcopy_tuple_with_mutable_rebuilt"
# subject = "copy.deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: deepcopy of a tuple containing a list returns a new tuple whose inner list is independent of the original"""
import copy

original = (1, [2, 3])
deep = copy.deepcopy(original)

# Mutating the original's inner list does not change the deep copy.
original[1].append(99)
assert original == (1, [2, 3, 99]), f"original inner list mutated: {original!r}"
assert deep == (1, [2, 3]), f"deep copy's inner list is independent: {deep!r}"

print("deepcopy_tuple_with_mutable_rebuilt OK")
"###);
    assert_output(&out, r###"deepcopy_tuple_with_mutable_rebuilt OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/pickle_getstate_setstate_drives_copy.py`.
#[test]
fn test_gen_behavior_std_libs_copy_pickle_getstate_setstate_drives_copy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "pickle_getstate_setstate_drives_copy"
# subject = "copy.deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: with no __copy__/__deepcopy__, copy falls back to the pickle protocol so __getstate__/__setstate__ drive the copied state and deepcopy makes it independent"""
import copy


# Plain instance: shallow keeps the same state object, deepcopy rebuilds it.
class Vanilla:
    def __init__(self, foo):
        self.foo = foo

    def __eq__(self, other):
        return self.foo == other.foo


v = Vanilla([42])
assert copy.copy(v) == v, "vanilla shallow equal"
vd = copy.deepcopy(v)
assert vd == v and vd.foo is not v.foo, "vanilla deepcopy independent state"


# __getstate__ + __setstate__ drive the copied state.
class StatePair:
    def __init__(self, foo):
        self.foo = foo

    def __getstate__(self):
        return self.foo

    def __setstate__(self, state):
        self.foo = state

    def __eq__(self, other):
        return self.foo == other.foo


sp = StatePair([42])
spd = copy.deepcopy(sp)
assert spd == sp and spd.foo is not sp.foo, "getstate/setstate deepcopy independent"

print("pickle_getstate_setstate_drives_copy OK")
"###);
    assert_output(&out, r###"pickle_getstate_setstate_drives_copy OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/reduce_ex_priority_string_returns_self.py`.
#[test]
fn test_gen_behavior_std_libs_copy_reduce_ex_priority_string_returns_self() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "reduce_ex_priority_string_returns_self"
# subject = "copy.copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.copy: __reduce_ex__ is consulted once with protocol 4 and takes priority over __reduce__; a string result means copy returns the object itself"""
import copy

calls = []


class ReduceEx:
    def __reduce_ex__(self, proto):
        calls.append(proto)
        return ""  # a string result -> copy returns the object itself

    def __reduce__(self):
        raise AssertionError("__reduce__ should not be consulted")


rx = ReduceEx()
assert copy.copy(rx) is rx, "reduce_ex string result returns self"
assert calls == [4], f"reduce_ex called once with protocol 4, got {calls!r}"

print("reduce_ex_priority_string_returns_self OK")
"###);
    assert_output(&out, r###"reduce_ex_priority_string_returns_self OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/reduce_string_returns_self.py`.
#[test]
fn test_gen_behavior_std_libs_copy_reduce_string_returns_self() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "reduce_string_returns_self"
# subject = "copy.copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.copy: a __reduce__ returning a string means both copy and deepcopy return the object itself"""
import copy


class GlobalRef:
    def __reduce__(self):
        return ""  # the global-reference pickle convention -> return self


g = GlobalRef()
assert copy.copy(g) is g, "reduce string result: copy returns self"
assert copy.deepcopy(g) is g, "reduce string result: deepcopy returns self"

print("reduce_string_returns_self OK")
"###);
    assert_output(&out, r###"reduce_string_returns_self OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/reduce_tuple_reconstructs_state.py`.
#[test]
fn test_gen_behavior_std_libs_copy_reduce_tuple_reconstructs_state() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "reduce_tuple_reconstructs_state"
# subject = "copy.deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: a __reduce__ 3-tuple (callable, args, state) reconstructs the instance, applying the state dict for copy and an independent copy of it for deepcopy"""
import copy


# __reduce__ returning (callable, args, state): copy applies the state dict,
# deepcopy applies an independent copy of it.
class Reconstruct:
    def __reduce__(self):
        return (Reconstruct, (), self.__dict__)

    def __eq__(self, other):
        return self.__dict__ == other.__dict__


r = Reconstruct()
r.foo = [42]
assert copy.copy(r) == r, "reduce 3-tuple shallow equal"
rd = copy.deepcopy(r)
assert rd == r and rd.foo is not r.foo, "reduce 3-tuple deepcopy independent state"


# A __reduce__ 2-tuple (no state) still reconstructs the class; an attribute set
# after construction is not reproduced.
class NoState:
    def __reduce__(self):
        return (NoState, ())


ns = NoState()
ns.foo = 42
assert copy.copy(ns).__class__ is ns.__class__, "reduce 2-tuple copy class"
assert copy.deepcopy(ns).__class__ is ns.__class__, "reduce 2-tuple deepcopy class"

print("reduce_tuple_reconstructs_state OK")
"###);
    assert_output(&out, r###"reduce_tuple_reconstructs_state OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/shallow_dict_shares_values.py`.
#[test]
fn test_gen_behavior_std_libs_copy_shallow_dict_shares_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "shallow_dict_shares_values"
# subject = "copy.copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.copy: shallow copy of a dict makes a new outer mapping but shares the mutable values"""
import copy

original = {"x": [1, 2], "y": [3, 4]}
shallow = copy.copy(original)
assert shallow == original and shallow is not original, "dict shallow outer is new"
assert shallow["x"] is original["x"], "dict shallow value is shared"

print("shallow_dict_shares_values OK")
"###);
    assert_output(&out, r###"shallow_dict_shares_values OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/shallow_list_shares_inner.py`.
#[test]
fn test_gen_behavior_std_libs_copy_shallow_list_shares_inner() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "shallow_list_shares_inner"
# subject = "copy.copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.copy: shallow copy of a nested list makes a new outer list but shares the inner sublists"""
import copy

original = [1, [2, 3], 4]
shallow = copy.copy(original)
assert shallow == original and shallow is not original, "shallow outer is new"
assert shallow[1] is original[1], "shallow inner sublist is shared"

print("shallow_list_shares_inner OK")
"###);
    assert_output(&out, r###"shallow_list_shares_inner OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/shallow_outer_mutation_isolated.py`.
#[test]
fn test_gen_behavior_std_libs_copy_shallow_outer_mutation_isolated() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "shallow_outer_mutation_isolated"
# subject = "copy.copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.copy: appending to a shallow-copied list does not change the original's length, but mutating a shared inner list is visible in both"""
import copy

original = [[1, 2], [3, 4]]
shallow = copy.copy(original)

# Appending to the shallow copy's outer list leaves the original's length alone.
shallow.append([5, 6])
assert len(original) == 2, f"original len unchanged = {len(original)!r}"
assert len(shallow) == 3, "shallow outer extended"

# But mutating a shared inner list is visible through the original.
shallow[0].append(99)
assert original[0] == [1, 2, 99], f"inner mutation visible in original: {original[0]!r}"

print("shallow_outer_mutation_isolated OK")
"###);
    assert_output(&out, r###"shallow_outer_mutation_isolated OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/slots_shallow_shares_deep_copies.py`.
#[test]
fn test_gen_behavior_std_libs_copy_slots_shallow_shares_deep_copies() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "slots_shallow_shares_deep_copies"
# subject = "copy.deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: a __slots__ instance shares its slot value under shallow copy and gets an independent slot value under deepcopy"""
import copy


class Slotted:
    __slots__ = ["foo"]


sl = Slotted()
sl.foo = [42]
assert copy.copy(sl).foo is sl.foo, "slots shallow shares the slot value"
sl_d = copy.deepcopy(sl)
assert sl_d.foo == sl.foo and sl_d.foo is not sl.foo, "slots deepcopy copies the slot value"

print("slots_shallow_shares_deep_copies OK")
"###);
    assert_output(&out, r###"slots_shallow_shares_deep_copies OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_copy_atomic.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_copy_atomic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_atomic"
# subject = "cpython.test_copy.TestCopy.test_copy_atomic"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_atomic
"""Auto-ported test: TestCopy::test_copy_atomic (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class NewStyle:
    pass

def f():
    pass

class WithMetaclass(metaclass=abc.ABCMeta):
    pass
tests = [None, ..., NotImplemented, 42, 2 ** 100, 3.14, True, False, 1j, 'hello', 'helloሴ', f.__code__, b'world', bytes(range(256)), range(10), slice(1, 10, 2), NewStyle, max, WithMetaclass, property()]
for x in tests:

    assert copy.copy(x) is x
print("TestCopy::test_copy_atomic: ok")
"###);
    assert_output(&out, r###"TestCopy::test_copy_atomic: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_copy_basic.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_copy_basic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_basic"
# subject = "cpython.test_copy.TestCopy.test_copy_basic"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_basic
"""Auto-ported test: TestCopy::test_copy_basic (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
x = 42
y = copy.copy(x)

assert x == y
print("TestCopy::test_copy_basic: ok")
"###);
    assert_output(&out, r###"TestCopy::test_copy_basic: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_copy_bytearray.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_copy_bytearray() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_bytearray"
# subject = "cpython.test_copy.TestCopy.test_copy_bytearray"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_bytearray
"""Auto-ported test: TestCopy::test_copy_bytearray (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
x = bytearray(b'abc')
y = copy.copy(x)

assert y == x

assert y is not x
x = bytearray()
y = copy.copy(x)

assert y == x

assert y is not x
print("TestCopy::test_copy_bytearray: ok")
"###);
    assert_output(&out, r###"TestCopy::test_copy_bytearray: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_copy_cant.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_copy_cant() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_cant"
# subject = "cpython.test_copy.TestCopy.test_copy_cant"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_cant
"""Auto-ported test: TestCopy::test_copy_cant (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C(object):

    def __getattribute__(self, name):
        if name.startswith('__reduce'):
            raise AttributeError(name)
        return object.__getattribute__(self, name)
x = C()

try:
    copy.copy(x)
    raise AssertionError('expected copy.Error')
except copy.Error:
    pass
print("TestCopy::test_copy_cant: ok")
"###);
    assert_output(&out, r###"TestCopy::test_copy_cant: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_copy_copy.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_copy_copy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_copy"
# subject = "cpython.test_copy.TestCopy.test_copy_copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_copy
"""Auto-ported test: TestCopy::test_copy_copy (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C(object):

    def __init__(self, foo):
        self.foo = foo

    def __copy__(self):
        return C(self.foo)
x = C(42)
y = copy.copy(x)

assert y.__class__ == x.__class__

assert y.foo == x.foo
print("TestCopy::test_copy_copy: ok")
"###);
    assert_output(&out, r###"TestCopy::test_copy_copy: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_copy_dict.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_copy_dict() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_dict"
# subject = "cpython.test_copy.TestCopy.test_copy_dict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_dict
"""Auto-ported test: TestCopy::test_copy_dict (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
x = {'foo': 1, 'bar': 2}
y = copy.copy(x)

assert y == x

assert y is not x
x = {}
y = copy.copy(x)

assert y == x

assert y is not x
print("TestCopy::test_copy_dict: ok")
"###);
    assert_output(&out, r###"TestCopy::test_copy_dict: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_copy_frozenset.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_copy_frozenset() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_frozenset"
# subject = "cpython.test_copy.TestCopy.test_copy_frozenset"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_frozenset
"""Auto-ported test: TestCopy::test_copy_frozenset (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
x = frozenset({1, 2, 3})

assert copy.copy(x) is x
x = frozenset()

assert copy.copy(x) is x
print("TestCopy::test_copy_frozenset: ok")
"###);
    assert_output(&out, r###"TestCopy::test_copy_frozenset: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_copy_function.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_copy_function() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_function"
# subject = "cpython.test_copy.TestCopy.test_copy_function"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_function
"""Auto-ported test: TestCopy::test_copy_function (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---

assert copy.copy(global_foo) == global_foo

def foo(x, y):
    return x + y

assert copy.copy(foo) == foo
bar = lambda: None

assert copy.copy(bar) == bar
print("TestCopy::test_copy_function: ok")
"###);
    assert_output(&out, r###"TestCopy::test_copy_function: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_copy_inst_copy.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_copy_inst_copy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_inst_copy"
# subject = "cpython.test_copy.TestCopy.test_copy_inst_copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_inst_copy
"""Auto-ported test: TestCopy::test_copy_inst_copy (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C:

    def __init__(self, foo):
        self.foo = foo

    def __copy__(self):
        return C(self.foo)

    def __eq__(self, other):
        return self.foo == other.foo
x = C(42)

assert copy.copy(x) == x
print("TestCopy::test_copy_inst_copy: ok")
"###);
    assert_output(&out, r###"TestCopy::test_copy_inst_copy: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_copy_inst_getinitargs.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_copy_inst_getinitargs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_inst_getinitargs"
# subject = "cpython.test_copy.TestCopy.test_copy_inst_getinitargs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_inst_getinitargs
"""Auto-ported test: TestCopy::test_copy_inst_getinitargs (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C:

    def __init__(self, foo):
        self.foo = foo

    def __getinitargs__(self):
        return (self.foo,)

    def __eq__(self, other):
        return self.foo == other.foo
x = C(42)

assert copy.copy(x) == x
print("TestCopy::test_copy_inst_getinitargs: ok")
"###);
    assert_output(&out, r###"TestCopy::test_copy_inst_getinitargs: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_copy_inst_getstate.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_copy_inst_getstate() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_inst_getstate"
# subject = "cpython.test_copy.TestCopy.test_copy_inst_getstate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_inst_getstate
"""Auto-ported test: TestCopy::test_copy_inst_getstate (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C:

    def __init__(self, foo):
        self.foo = foo

    def __getstate__(self):
        return {'foo': self.foo}

    def __eq__(self, other):
        return self.foo == other.foo
x = C(42)

assert copy.copy(x) == x
print("TestCopy::test_copy_inst_getstate: ok")
"###);
    assert_output(&out, r###"TestCopy::test_copy_inst_getstate: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_copy_inst_getstate_setstate.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_copy_inst_getstate_setstate() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_inst_getstate_setstate"
# subject = "cpython.test_copy.TestCopy.test_copy_inst_getstate_setstate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_inst_getstate_setstate
"""Auto-ported test: TestCopy::test_copy_inst_getstate_setstate (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C:

    def __init__(self, foo):
        self.foo = foo

    def __getstate__(self):
        return self.foo

    def __setstate__(self, state):
        self.foo = state

    def __eq__(self, other):
        return self.foo == other.foo
x = C(42)

assert copy.copy(x) == x
x = C(0.0)

assert copy.copy(x) == x
print("TestCopy::test_copy_inst_getstate_setstate: ok")
"###);
    assert_output(&out, r###"TestCopy::test_copy_inst_getstate_setstate: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_copy_inst_setstate.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_copy_inst_setstate() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_inst_setstate"
# subject = "cpython.test_copy.TestCopy.test_copy_inst_setstate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_inst_setstate
"""Auto-ported test: TestCopy::test_copy_inst_setstate (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C:

    def __init__(self, foo):
        self.foo = foo

    def __setstate__(self, state):
        self.foo = state['foo']

    def __eq__(self, other):
        return self.foo == other.foo
x = C(42)

assert copy.copy(x) == x
print("TestCopy::test_copy_inst_setstate: ok")
"###);
    assert_output(&out, r###"TestCopy::test_copy_inst_setstate: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_copy_inst_vanilla.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_copy_inst_vanilla() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_inst_vanilla"
# subject = "cpython.test_copy.TestCopy.test_copy_inst_vanilla"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_inst_vanilla
"""Auto-ported test: TestCopy::test_copy_inst_vanilla (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C:

    def __init__(self, foo):
        self.foo = foo

    def __eq__(self, other):
        return self.foo == other.foo
x = C(42)

assert copy.copy(x) == x
print("TestCopy::test_copy_inst_vanilla: ok")
"###);
    assert_output(&out, r###"TestCopy::test_copy_inst_vanilla: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_copy_list.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_copy_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_list"
# subject = "cpython.test_copy.TestCopy.test_copy_list"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_list
"""Auto-ported test: TestCopy::test_copy_list (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
x = [1, 2, 3]
y = copy.copy(x)

assert y == x

assert y is not x
x = []
y = copy.copy(x)

assert y == x

assert y is not x
print("TestCopy::test_copy_list: ok")
"###);
    assert_output(&out, r###"TestCopy::test_copy_list: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_copy_reduce.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_copy_reduce() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_reduce"
# subject = "cpython.test_copy.TestCopy.test_copy_reduce"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_reduce
"""Auto-ported test: TestCopy::test_copy_reduce (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C(object):

    def __reduce__(self):
        c.append(1)
        return ''
c = []
x = C()
y = copy.copy(x)

assert y is x

assert c == [1]
print("TestCopy::test_copy_reduce: ok")
"###);
    assert_output(&out, r###"TestCopy::test_copy_reduce: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_copy_reduce_ex.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_copy_reduce_ex() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_reduce_ex"
# subject = "cpython.test_copy.TestCopy.test_copy_reduce_ex"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_reduce_ex
"""Auto-ported test: TestCopy::test_copy_reduce_ex (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C(object):

    def __reduce_ex__(self, proto):
        c.append(1)
        return ''

    def __reduce__(self):
        self.fail("shouldn't call this")
c = []
x = C()
y = copy.copy(x)

assert y is x

assert c == [1]
print("TestCopy::test_copy_reduce_ex: ok")
"###);
    assert_output(&out, r###"TestCopy::test_copy_reduce_ex: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_copy_set.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_copy_set() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_set"
# subject = "cpython.test_copy.TestCopy.test_copy_set"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_set
"""Auto-ported test: TestCopy::test_copy_set (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
x = {1, 2, 3}
y = copy.copy(x)

assert y == x

assert y is not x
x = set()
y = copy.copy(x)

assert y == x

assert y is not x
print("TestCopy::test_copy_set: ok")
"###);
    assert_output(&out, r###"TestCopy::test_copy_set: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_copy_slots.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_copy_slots() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_slots"
# subject = "cpython.test_copy.TestCopy.test_copy_slots"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_slots
"""Auto-ported test: TestCopy::test_copy_slots (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C(object):
    __slots__ = ['foo']
x = C()
x.foo = [42]
y = copy.copy(x)

assert x.foo is y.foo
print("TestCopy::test_copy_slots: ok")
"###);
    assert_output(&out, r###"TestCopy::test_copy_slots: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_copy_tuple.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_copy_tuple() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_tuple"
# subject = "cpython.test_copy.TestCopy.test_copy_tuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_tuple
"""Auto-ported test: TestCopy::test_copy_tuple (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
x = (1, 2, 3)

assert copy.copy(x) is x
x = ()

assert copy.copy(x) is x
x = (1, 2, 3, [])

assert copy.copy(x) is x
print("TestCopy::test_copy_tuple: ok")
"###);
    assert_output(&out, r###"TestCopy::test_copy_tuple: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_copy_weakref.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_copy_weakref() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_weakref"
# subject = "cpython.test_copy.TestCopy.test_copy_weakref"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_weakref
"""Auto-ported test: TestCopy::test_copy_weakref (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
def _check_copy_weakdict(_dicttype):

    class C(object):
        pass
    a, b, c, d = [C() for i in range(4)]
    u = _dicttype()
    u[a] = b
    u[c] = d
    v = copy.copy(u)

    assert v is not u

    assert v == u

    assert v[a] == b

    assert v[c] == d

    assert len(v) == 2
    del c, d
    support.gc_collect()

    assert len(v) == 1
    x, y = (C(), C())
    v[x] = y

    assert x not in u

def _check_weakref(_copy):

    class C(object):
        pass
    obj = C()
    x = weakref.ref(obj)
    y = _copy(x)

    assert y is x
    del obj
    y = _copy(x)

    assert y is x
_check_weakref(copy.copy)
print("TestCopy::test_copy_weakref: ok")
"###);
    assert_output(&out, r###"TestCopy::test_copy_weakref: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_atomic.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_atomic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_atomic"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_atomic"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_atomic
"""Auto-ported test: TestCopy::test_deepcopy_atomic (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class NewStyle:
    pass

def f():
    pass
tests = [None, ..., NotImplemented, 42, 2 ** 100, 3.14, True, False, 1j, b'bytes', 'hello', 'helloሴ', f.__code__, NewStyle, range(10), max, property()]
for x in tests:

    assert copy.deepcopy(x) is x
print("TestCopy::test_deepcopy_atomic: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_atomic: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_basic.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_basic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_basic"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_basic"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_basic
"""Auto-ported test: TestCopy::test_deepcopy_basic (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
x = 42
y = copy.deepcopy(x)

assert y == x
print("TestCopy::test_deepcopy_basic: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_basic: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_cant.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_cant() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_cant"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_cant"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_cant
"""Auto-ported test: TestCopy::test_deepcopy_cant (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C(object):

    def __getattribute__(self, name):
        if name.startswith('__reduce'):
            raise AttributeError(name)
        return object.__getattribute__(self, name)
x = C()

try:
    copy.deepcopy(x)
    raise AssertionError('expected copy.Error')
except copy.Error:
    pass
print("TestCopy::test_deepcopy_cant: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_cant: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_deepcopy.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_deepcopy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_deepcopy"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_deepcopy
"""Auto-ported test: TestCopy::test_deepcopy_deepcopy (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C(object):

    def __init__(self, foo):
        self.foo = foo

    def __deepcopy__(self, memo=None):
        return C(self.foo)
x = C(42)
y = copy.deepcopy(x)

assert y.__class__ == x.__class__

assert y.foo == x.foo
print("TestCopy::test_deepcopy_deepcopy: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_deepcopy: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_dict.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_dict() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_dict"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_dict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_dict
"""Auto-ported test: TestCopy::test_deepcopy_dict (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
x = {'foo': [1, 2], 'bar': 3}
y = copy.deepcopy(x)

assert y == x

assert x is not y

assert x['foo'] is not y['foo']
print("TestCopy::test_deepcopy_dict: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_dict: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_dont_memo_immutable.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_dont_memo_immutable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_dont_memo_immutable"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_dont_memo_immutable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_dont_memo_immutable
"""Auto-ported test: TestCopy::test_deepcopy_dont_memo_immutable (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
memo = {}
x = [1, 2, 3, 4]
y = copy.deepcopy(x, memo)

assert y == x

assert len(memo) == 2
memo = {}
x = [(1, 2)]
y = copy.deepcopy(x, memo)

assert y == x

assert len(memo) == 2
print("TestCopy::test_deepcopy_dont_memo_immutable: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_dont_memo_immutable: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_empty_tuple.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_empty_tuple() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_empty_tuple"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_empty_tuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_empty_tuple
"""Auto-ported test: TestCopy::test_deepcopy_empty_tuple (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
x = ()
y = copy.deepcopy(x)

assert x is y
print("TestCopy::test_deepcopy_empty_tuple: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_empty_tuple: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_function.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_function() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_function"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_function"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_function
"""Auto-ported test: TestCopy::test_deepcopy_function (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---

assert copy.deepcopy(global_foo) == global_foo

def foo(x, y):
    return x + y

assert copy.deepcopy(foo) == foo
bar = lambda: None

assert copy.deepcopy(bar) == bar
print("TestCopy::test_deepcopy_function: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_function: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_inst_deepcopy.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_inst_deepcopy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_inst_deepcopy"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_inst_deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_inst_deepcopy
"""Auto-ported test: TestCopy::test_deepcopy_inst_deepcopy (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C:

    def __init__(self, foo):
        self.foo = foo

    def __deepcopy__(self, memo):
        return C(copy.deepcopy(self.foo, memo))

    def __eq__(self, other):
        return self.foo == other.foo
x = C([42])
y = copy.deepcopy(x)

assert y == x

assert y is not x

assert y.foo is not x.foo
print("TestCopy::test_deepcopy_inst_deepcopy: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_inst_deepcopy: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_inst_getinitargs.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_inst_getinitargs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_inst_getinitargs"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_inst_getinitargs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_inst_getinitargs
"""Auto-ported test: TestCopy::test_deepcopy_inst_getinitargs (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C:

    def __init__(self, foo):
        self.foo = foo

    def __getinitargs__(self):
        return (self.foo,)

    def __eq__(self, other):
        return self.foo == other.foo
x = C([42])
y = copy.deepcopy(x)

assert y == x

assert y is not x

assert y.foo is not x.foo
print("TestCopy::test_deepcopy_inst_getinitargs: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_inst_getinitargs: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_inst_getstate.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_inst_getstate() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_inst_getstate"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_inst_getstate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_inst_getstate
"""Auto-ported test: TestCopy::test_deepcopy_inst_getstate (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C:

    def __init__(self, foo):
        self.foo = foo

    def __getstate__(self):
        return {'foo': self.foo}

    def __eq__(self, other):
        return self.foo == other.foo
x = C([42])
y = copy.deepcopy(x)

assert y == x

assert y is not x

assert y.foo is not x.foo
print("TestCopy::test_deepcopy_inst_getstate: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_inst_getstate: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_inst_getstate_setstate.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_inst_getstate_setstate() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_inst_getstate_setstate"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_inst_getstate_setstate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_inst_getstate_setstate
"""Auto-ported test: TestCopy::test_deepcopy_inst_getstate_setstate (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C:

    def __init__(self, foo):
        self.foo = foo

    def __getstate__(self):
        return self.foo

    def __setstate__(self, state):
        self.foo = state

    def __eq__(self, other):
        return self.foo == other.foo
x = C([42])
y = copy.deepcopy(x)

assert y == x

assert y is not x

assert y.foo is not x.foo
x = C([])
y = copy.deepcopy(x)

assert y == x

assert y is not x

assert y.foo is not x.foo
print("TestCopy::test_deepcopy_inst_getstate_setstate: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_inst_getstate_setstate: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_inst_setstate.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_inst_setstate() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_inst_setstate"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_inst_setstate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_inst_setstate
"""Auto-ported test: TestCopy::test_deepcopy_inst_setstate (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C:

    def __init__(self, foo):
        self.foo = foo

    def __setstate__(self, state):
        self.foo = state['foo']

    def __eq__(self, other):
        return self.foo == other.foo
x = C([42])
y = copy.deepcopy(x)

assert y == x

assert y is not x

assert y.foo is not x.foo
print("TestCopy::test_deepcopy_inst_setstate: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_inst_setstate: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_inst_vanilla.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_inst_vanilla() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_inst_vanilla"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_inst_vanilla"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_inst_vanilla
"""Auto-ported test: TestCopy::test_deepcopy_inst_vanilla (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C:

    def __init__(self, foo):
        self.foo = foo

    def __eq__(self, other):
        return self.foo == other.foo
x = C([42])
y = copy.deepcopy(x)

assert y == x

assert y.foo is not x.foo
print("TestCopy::test_deepcopy_inst_vanilla: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_inst_vanilla: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_issubclass.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_issubclass() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_issubclass"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_issubclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_issubclass
"""Auto-ported test: TestCopy::test_deepcopy_issubclass (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class Meta(type):
    pass

class C(metaclass=Meta):
    pass

assert copy.deepcopy(C) == C
print("TestCopy::test_deepcopy_issubclass: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_issubclass: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_keepalive.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_keepalive() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_keepalive"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_keepalive"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_keepalive
"""Auto-ported test: TestCopy::test_deepcopy_keepalive (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
memo = {}
x = []
y = copy.deepcopy(x, memo)

assert memo[id(memo)][0] is x
print("TestCopy::test_deepcopy_keepalive: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_keepalive: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_list.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_list"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_list"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_list
"""Auto-ported test: TestCopy::test_deepcopy_list (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
x = [[1, 2], 3]
y = copy.deepcopy(x)

assert y == x

assert x is not y

assert x[0] is not y[0]
print("TestCopy::test_deepcopy_list: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_list: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_memo.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_memo() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_memo"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_memo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_memo
"""Auto-ported test: TestCopy::test_deepcopy_memo (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
x = []
x = [x, x]
y = copy.deepcopy(x)

assert y == x

assert y is not x

assert y[0] is not x[0]

assert y[0] is y[1]
print("TestCopy::test_deepcopy_memo: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_memo: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_reduce.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_reduce() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_reduce"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_reduce"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_reduce
"""Auto-ported test: TestCopy::test_deepcopy_reduce (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C(object):

    def __reduce__(self):
        c.append(1)
        return ''
c = []
x = C()
y = copy.deepcopy(x)

assert y is x

assert c == [1]
print("TestCopy::test_deepcopy_reduce: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_reduce: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_reduce_ex.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_reduce_ex() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_reduce_ex"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_reduce_ex"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_reduce_ex
"""Auto-ported test: TestCopy::test_deepcopy_reduce_ex (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C(object):

    def __reduce_ex__(self, proto):
        c.append(1)
        return ''

    def __reduce__(self):
        self.fail("shouldn't call this")
c = []
x = C()
y = copy.deepcopy(x)

assert y is x

assert c == [1]
print("TestCopy::test_deepcopy_reduce_ex: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_reduce_ex: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_reflexive_inst.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_reflexive_inst() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_reflexive_inst"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_reflexive_inst"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_reflexive_inst
"""Auto-ported test: TestCopy::test_deepcopy_reflexive_inst (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C:
    pass
x = C()
x.foo = x
y = copy.deepcopy(x)

assert y is not x

assert y.foo is y
print("TestCopy::test_deepcopy_reflexive_inst: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_reflexive_inst: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_slots.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_slots() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_slots"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_slots"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_slots
"""Auto-ported test: TestCopy::test_deepcopy_slots (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C(object):
    __slots__ = ['foo']
x = C()
x.foo = [42]
y = copy.deepcopy(x)

assert x.foo == y.foo

assert x.foo is not y.foo
print("TestCopy::test_deepcopy_slots: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_slots: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_tuple.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_tuple() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_tuple"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_tuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_tuple
"""Auto-ported test: TestCopy::test_deepcopy_tuple (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
x = ([1, 2], 3)
y = copy.deepcopy(x)

assert y == x

assert x is not y

assert x[0] is not y[0]
print("TestCopy::test_deepcopy_tuple: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_tuple: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_tuple_of_immutables.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_tuple_of_immutables() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_tuple_of_immutables"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_tuple_of_immutables"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_tuple_of_immutables
"""Auto-ported test: TestCopy::test_deepcopy_tuple_of_immutables (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
x = ((1, 2), 3)
y = copy.deepcopy(x)

assert x is y
print("TestCopy::test_deepcopy_tuple_of_immutables: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_tuple_of_immutables: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_deepcopy_weakref.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_deepcopy_weakref() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_weakref"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_weakref"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_weakref
"""Auto-ported test: TestCopy::test_deepcopy_weakref (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
def _check_copy_weakdict(_dicttype):

    class C(object):
        pass
    a, b, c, d = [C() for i in range(4)]
    u = _dicttype()
    u[a] = b
    u[c] = d
    v = copy.copy(u)

    assert v is not u

    assert v == u

    assert v[a] == b

    assert v[c] == d

    assert len(v) == 2
    del c, d
    support.gc_collect()

    assert len(v) == 1
    x, y = (C(), C())
    v[x] = y

    assert x not in u

def _check_weakref(_copy):

    class C(object):
        pass
    obj = C()
    x = weakref.ref(obj)
    y = _copy(x)

    assert y is x
    del obj
    y = _copy(x)

    assert y is x
_check_weakref(copy.deepcopy)
print("TestCopy::test_deepcopy_weakref: ok")
"###);
    assert_output(&out, r###"TestCopy::test_deepcopy_weakref: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_exceptions.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_exceptions() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_exceptions"
# subject = "cpython.test_copy.TestCopy.test_exceptions"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_exceptions
"""Auto-ported test: TestCopy::test_exceptions (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---

assert copy.Error is copy.error

assert issubclass(copy.Error, Exception)
print("TestCopy::test_exceptions: ok")
"###);
    assert_output(&out, r###"TestCopy::test_exceptions: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_getstate_exc.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_getstate_exc() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_getstate_exc"
# subject = "cpython.test_copy.TestCopy.test_getstate_exc"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_getstate_exc
"""Auto-ported test: TestCopy::test_getstate_exc (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class EvilState(object):

    def __getstate__(self):
        raise ValueError("ain't got no stickin' state")

try:
    copy.copy(EvilState())
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("TestCopy::test_getstate_exc: ok")
"###);
    assert_output(&out, r###"TestCopy::test_getstate_exc: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_reconstruct_nostate.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_reconstruct_nostate() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_reconstruct_nostate"
# subject = "cpython.test_copy.TestCopy.test_reconstruct_nostate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_reconstruct_nostate
"""Auto-ported test: TestCopy::test_reconstruct_nostate (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C(object):

    def __reduce__(self):
        return (C, ())
x = C()
x.foo = 42
y = copy.copy(x)

assert y.__class__ is x.__class__
y = copy.deepcopy(x)

assert y.__class__ is x.__class__
print("TestCopy::test_reconstruct_nostate: ok")
"###);
    assert_output(&out, r###"TestCopy::test_reconstruct_nostate: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_reconstruct_reflexive.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_reconstruct_reflexive() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_reconstruct_reflexive"
# subject = "cpython.test_copy.TestCopy.test_reconstruct_reflexive"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_reconstruct_reflexive
"""Auto-ported test: TestCopy::test_reconstruct_reflexive (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C(object):
    pass
x = C()
x.foo = x
y = copy.deepcopy(x)

assert y is not x

assert y.foo is y
print("TestCopy::test_reconstruct_reflexive: ok")
"###);
    assert_output(&out, r###"TestCopy::test_reconstruct_reflexive: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_reconstruct_state.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_reconstruct_state() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_reconstruct_state"
# subject = "cpython.test_copy.TestCopy.test_reconstruct_state"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_reconstruct_state
"""Auto-ported test: TestCopy::test_reconstruct_state (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C(object):

    def __reduce__(self):
        return (C, (), self.__dict__)

    def __eq__(self, other):
        return self.__dict__ == other.__dict__
x = C()
x.foo = [42]
y = copy.copy(x)

assert y == x
y = copy.deepcopy(x)

assert y == x

assert y.foo is not x.foo
print("TestCopy::test_reconstruct_state: ok")
"###);
    assert_output(&out, r###"TestCopy::test_reconstruct_state: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_reconstruct_string.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_reconstruct_string() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_reconstruct_string"
# subject = "cpython.test_copy.TestCopy.test_reconstruct_string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_reconstruct_string
"""Auto-ported test: TestCopy::test_reconstruct_string (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C(object):

    def __reduce__(self):
        return ''
x = C()
y = copy.copy(x)

assert y is x
y = copy.deepcopy(x)

assert y is x
print("TestCopy::test_reconstruct_string: ok")
"###);
    assert_output(&out, r###"TestCopy::test_reconstruct_string: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_reduce_6tuple.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_reduce_6tuple() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_reduce_6tuple"
# subject = "cpython.test_copy.TestCopy.test_reduce_6tuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_reduce_6tuple
"""Auto-ported test: TestCopy::test_reduce_6tuple (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
def state_setter(*args, **kwargs):
    self.fail("shouldn't call this")

class C:

    def __reduce__(self):
        return (C, (), self.__dict__, None, None, state_setter)
x = C()
try:
    copy.copy(x)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    copy.deepcopy(x)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestCopy::test_reduce_6tuple: ok")
"###);
    assert_output(&out, r###"TestCopy::test_reduce_6tuple: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/copy/test_copy__test_reduce_6tuple_none.py`.
#[test]
fn test_gen_behavior_std_libs_copy_test_copy__test_reduce_6tuple_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_reduce_6tuple_none"
# subject = "cpython.test_copy.TestCopy.test_reduce_6tuple_none"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_reduce_6tuple_none
"""Auto-ported test: TestCopy::test_reduce_6tuple_none (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
class C:

    def __reduce__(self):
        return (C, (), self.__dict__, None, None, None)
x = C()
try:
    copy.copy(x)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    copy.deepcopy(x)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestCopy::test_reduce_6tuple_none: ok")
"###);
    assert_output(&out, r###"TestCopy::test_reduce_6tuple_none: ok
"###);
}
