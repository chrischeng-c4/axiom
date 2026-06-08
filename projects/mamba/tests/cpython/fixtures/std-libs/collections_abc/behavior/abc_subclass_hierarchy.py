# /// script
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
