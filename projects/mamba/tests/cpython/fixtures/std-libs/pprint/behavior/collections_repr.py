# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "behavior"
# case = "collections_repr"
# subject = "pprint.pformat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.pformat: pprint keeps each container's repr prefix (Counter/OrderedDict/deque/ChainMap/defaultdict/mappingproxy, deque maxlen=) while wrapping contents, and User* wrappers format like their builtin"""
import collections
import itertools
import types

import pprint

words = "the quick brown fox jumped over a lazy dog".split()

# Empty containers keep their type prefix even at width=1.
assert pprint.pformat(collections.Counter(), width=1) == "Counter()"
assert pprint.pformat(collections.OrderedDict(), width=1) == "OrderedDict()"
assert pprint.pformat(collections.deque(), width=1) == "deque([])"
assert pprint.pformat(collections.ChainMap(), width=1) == "ChainMap({})"
assert pprint.pformat(collections.defaultdict(int), width=1) == \
    "defaultdict(<class 'int'>, {})"

# Counter wraps and keeps insertion (count-descending) order, not sorted.
c = collections.Counter("senselessness")
assert pprint.pformat(c, width=40) == (
    "Counter({'s': 6,\n         'e': 4,\n         'n': 2,\n         'l': 1})"
)

# deque with a maxlen surfaces the maxlen= keyword in the repr.
dq = collections.deque(zip(words, itertools.count()), maxlen=7)
assert pprint.pformat(dq) == (
    "deque([('brown', 2),\n       ('fox', 3),\n       ('jumped', 4),\n"
    "       ('over', 5),\n       ('a', 6),\n       ('lazy', 7),\n"
    "       ('dog', 8)],\n      maxlen=7)"
)

# OrderedDict preserves insertion order inside an OrderedDict({...}) wrap.
od = collections.OrderedDict(zip(words[:3], itertools.count()))
assert pprint.pformat(od) == \
    "OrderedDict({'the': 0, 'quick': 1, 'brown': 2})"

# mappingproxy delegates to the underlying mapping (here keeping its order).
mp = types.MappingProxyType({"b": 2, "a": 1})
assert pprint.pformat(mp) == "mappingproxy({'b': 2, 'a': 1})"

# User* wrappers format identically to the builtin they emulate.
assert pprint.pformat(collections.UserDict(), width=1) == "{}"
assert pprint.pformat(collections.UserList(), width=1) == "[]"
assert pprint.pformat(collections.UserString(""), width=1) == "''"
assert pprint.pformat(collections.UserDict({"a": 1})) == "{'a': 1}"
print("collections_repr OK")
