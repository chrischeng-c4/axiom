# /// script
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
