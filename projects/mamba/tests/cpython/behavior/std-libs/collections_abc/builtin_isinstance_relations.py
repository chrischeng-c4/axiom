# /// script
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
