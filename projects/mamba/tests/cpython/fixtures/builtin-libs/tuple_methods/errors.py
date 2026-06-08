# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""tuple methods: documented exception paths (CPython 3.12 oracle)."""


# Indexing out of range raises IndexError.
try:
    (1, 2)[5]
    print("oor: no_raise")
except IndexError as e:
    print("oor:", type(e).__name__, str(e)[:60])


# Indexing with a non-integer key raises TypeError, not IndexError.
empty = ()
key = "a"
try:
    empty[key]  # type: ignore[index]
    print("str_index: no_raise")
except TypeError as e:
    print("str_index:", type(e).__name__, "tuple indices" in str(e))


# index of missing element raises ValueError.
try:
    (1, 2).index(99)
    print("missing_index: no_raise")
except ValueError as e:
    print("missing_index:", type(e).__name__, str(e)[:60])


# Tuple is immutable — assignment raises TypeError.
t = (1, 2, 3)
try:
    t[0] = 99  # type: ignore[index]
    print("set_item: no_raise")
except TypeError as e:
    print("set_item:", type(e).__name__, str(e)[:60])


# del item raises TypeError.
try:
    del t[0]  # type: ignore[misc]
    print("del_item: no_raise")
except TypeError as e:
    print("del_item:", type(e).__name__, str(e)[:60])


# Concatenating tuple + non-tuple raises TypeError.
try:
    (1, 2) + [3]  # type: ignore[operator]
    print("plus_list: no_raise")
except TypeError as e:
    print("plus_list:", type(e).__name__, str(e)[:60])


# Multiplying tuple by non-int raises TypeError.
try:
    (1,) * "x"  # type: ignore[operator]
    print("times_str: no_raise")
except TypeError as e:
    print("times_str:", type(e).__name__, str(e)[:60])


# tuple() from non-iterable raises TypeError.
try:
    tuple(42)  # type: ignore[arg-type]
    print("from_int: no_raise")
except TypeError as e:
    print("from_int:", type(e).__name__, str(e)[:60])


# Unhashable element makes the tuple unhashable.
try:
    hash(([1, 2], 3))  # type: ignore[arg-type]
    print("hash_with_list: no_raise")
except TypeError as e:
    print("hash_with_list:", type(e).__name__, str(e)[:60])


# tuple() takes no keyword arguments.
try:
    tuple(sequence=())  # type: ignore[call-overload]
    print("kwarg: no_raise")
except TypeError as e:
    print("kwarg:", type(e).__name__, "keyword argument" in str(e))


# list + tuple-subclass is still a list/tuple mismatch and raises TypeError;
# the subclass does not make the left operand accept it.
class TupleSub(tuple):
    pass


try:
    [3] + TupleSub((1, 2))  # type: ignore[operator]
    print("list_plus_sub: no_raise")
except TypeError as e:
    print("list_plus_sub:", type(e).__name__, str(e)[:60])
