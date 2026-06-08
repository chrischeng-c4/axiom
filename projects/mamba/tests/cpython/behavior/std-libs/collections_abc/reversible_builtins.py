# /// script
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
