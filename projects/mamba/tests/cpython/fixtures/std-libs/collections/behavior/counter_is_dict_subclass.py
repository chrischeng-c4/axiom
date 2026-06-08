# /// script
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
