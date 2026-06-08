# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""list methods: documented exception paths (CPython 3.12 oracle)."""


# pop from empty list raises IndexError.
try:
    [].pop()
    print("pop_empty: no_raise")
except IndexError as e:
    print("pop_empty:", type(e).__name__, str(e)[:60])


# pop with out-of-range index raises IndexError.
try:
    [1, 2].pop(5)
    print("pop_oor: no_raise")
except IndexError as e:
    print("pop_oor:", type(e).__name__, str(e)[:60])


# index of missing element raises ValueError.
try:
    [1, 2].index(99)
    print("missing_index: no_raise")
except ValueError as e:
    print("missing_index:", type(e).__name__, str(e)[:60])


# remove of missing element raises ValueError.
try:
    [1, 2].remove(99)
    print("missing_remove: no_raise")
except ValueError as e:
    print("missing_remove:", type(e).__name__, str(e)[:60])


# Indexing out of range raises IndexError.
try:
    [1, 2][5]
    print("oor: no_raise")
except IndexError as e:
    print("oor:", type(e).__name__, str(e)[:60])


# Concatenating list + non-list raises TypeError.
try:
    [1] + "abc"  # type: ignore[operator]
    print("plus_str: no_raise")
except TypeError as e:
    print("plus_str:", type(e).__name__, str(e)[:60])


# Multiplying list by non-int raises TypeError.
try:
    [1] * "x"  # type: ignore[operator]
    print("times_str: no_raise")
except TypeError as e:
    print("times_str:", type(e).__name__, str(e)[:60])


# sort with non-callable key raises TypeError.
try:
    [1, 2].sort(key=42)  # type: ignore[arg-type]
    print("sort_bad_key: no_raise")
except TypeError as e:
    print("sort_bad_key:", type(e).__name__, str(e)[:60])


# sort with mixed types raises TypeError.
try:
    [1, "two"].sort()  # type: ignore[list-item]
    print("sort_mixed: no_raise")
except TypeError as e:
    print("sort_mixed:", type(e).__name__, str(e)[:60])


# list() takes no keyword arguments.
try:
    list(sequence=[])  # type: ignore[call-overload]
    print("list_kwarg: no_raise")
except TypeError as e:
    print("list_kwarg:", type(e).__name__, str(e)[:60])


# tuple + list (even a list subclass) is not concatenable: TypeError.
class _ListSub(list):
    pass


try:
    (3,) + _ListSub([1, 2])  # type: ignore[operator]
    print("tuple_plus_list: no_raise")
except TypeError as e:
    print("tuple_plus_list:", type(e).__name__, str(e)[:60])


# Assigning a wrong-size sequence to an extended slice raises ValueError.
try:
    target = [0, 1, 2, 3, 4]
    target[::2] = [9, 9]  # extended slice expects exactly 3 items
    print("extended_slice_size: no_raise")
except ValueError as e:
    print("extended_slice_size:", type(e).__name__, str(e)[:60])
