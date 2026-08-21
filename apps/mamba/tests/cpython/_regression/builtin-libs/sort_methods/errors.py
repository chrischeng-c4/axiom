# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""sort methods: documented exception paths (CPython 3.12 oracle)."""


# Mixed-type sort raises TypeError.
try:
    sorted([1, "two", 3])  # type: ignore[type-var]
    print("mixed_sorted: no_raise")
except TypeError as e:
    print("mixed_sorted:", type(e).__name__, str(e)[:60])


# list.sort() with non-callable key raises TypeError.
try:
    [1, 2, 3].sort(key=42)  # type: ignore[arg-type]
    print("sort_bad_key: no_raise")
except TypeError as e:
    print("sort_bad_key:", type(e).__name__, str(e)[:60])


# sorted with key that raises propagates.
def boom(_x):
    raise ValueError("custom key failure")


try:
    sorted([1, 2, 3], key=boom)
    print("key_raises: no_raise")
except ValueError as e:
    print("key_raises:", type(e).__name__, str(e)[:60])


# Sorting a non-iterable raises TypeError.
try:
    sorted(42)  # type: ignore[arg-type]
    print("non_iter: no_raise")
except TypeError as e:
    print("non_iter:", type(e).__name__, str(e)[:60])


# Reverse=non-bool: TypeError.
try:
    sorted([1, 2], reverse="not bool")  # type: ignore[arg-type]
    print("bad_reverse: no_raise")
except TypeError as e:
    print("bad_reverse:", type(e).__name__, str(e)[:60])


# Stable sort: keys that compare equal preserve original order.
items = [("a", 1), ("b", 2), ("a", 3)]
stable = sorted(items, key=lambda t: t[0])
print("stable:", stable)


# Happy: sorted with reverse + key.
print("by_len:", sorted(["aa", "b", "cccc"], key=len, reverse=True))


# A callable key with wrong arity (expects 2 args) raises TypeError when
# the sort calls it with a single element.
try:
    ["b", "a", "c"].sort(key=lambda x, y: 0)  # type: ignore[misc]
    print("two_arg_key: no_raise")
except TypeError as e:
    print("two_arg_key:", type(e).__name__, str(e)[:60])


# An exception raised inside the key function leaves the list unchanged.
original = list(range(-2, 2))
target = original[:]
try:
    target.sort(key=lambda x: 1 / x)  # ZeroDivisionError at x == 0
    print("key_exc: no_raise")
except ZeroDivisionError as e:
    print("key_exc:", type(e).__name__, "unchanged=", target == [-2, -1, 0, 1])


# A list mixing tuples and a bare int is not all-tuples: comparison falls
# through to comparing a tuple against an int and raises TypeError.
try:
    [(1.0, 1.0), (False, "A"), 6].sort()  # type: ignore[list-item]
    print("mixed_tuple_int: no_raise")
except TypeError as e:
    print("mixed_tuple_int:", type(e).__name__, str(e)[:40])


# Tuples whose first elements are comparable but second elements are not
# (str vs int) raise TypeError once the tie-break reaches the bad pair.
try:
    [("a", 1), ("a", "z")].sort()  # type: ignore[list-item]
    print("tuple_tiebreak_badtype: no_raise")
except TypeError as e:
    print("tuple_tiebreak_badtype:", type(e).__name__, str(e)[:40])
