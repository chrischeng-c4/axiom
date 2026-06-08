# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "cmp_to_key_orders_and_sorts"
# subject = "functools.cmp_to_key"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.cmp_to_key: cmp_to_key turns an old-style cmp into key objects that compare via the cmp, drive sorted(), and are not hashable"""
import collections.abc
import functools

cmp_to_key = functools.cmp_to_key


def _cmp(x, y):
    return (x > y) - (x < y)


# Key objects compare against each other using the underlying cmp.
key = cmp_to_key(_cmp)
assert key(3) == key(3), "equal keys"
assert key(3) > key(1), "greater key"
assert key(3) >= key(3), "ge key"
assert key(1) < key(3), "lesser key"
assert key(2) != key(5), "unequal keys"


# A cmp that coerces to int lets mixed int/str values compare and sort.
def _cmp_as_int(x, y):
    x, y = int(x), int(y)
    return (x > y) - (x < y)


mixed_key = cmp_to_key(_cmp_as_int)
assert mixed_key(4) == mixed_key("4"), "int vs str equal"
assert mixed_key(2) < mixed_key("35"), "int vs str less"

values = [5, "3", 7, 2, "0", "1", 4, "10", 1]
ordered = sorted(values, key=cmp_to_key(_cmp_as_int))
assert [int(v) for v in ordered] == [0, 1, 1, 2, 3, 4, 5, 7, 10], (
    f"sorted = {ordered!r}"
)


# Key objects are not hashable.
k = cmp_to_key(_cmp)(10)
try:
    hash(k)
    raise AssertionError("expected TypeError on hash(key)")
except TypeError:
    pass
assert not isinstance(k, collections.abc.Hashable), "key not Hashable"

print("cmp_to_key_orders_and_sorts OK")
