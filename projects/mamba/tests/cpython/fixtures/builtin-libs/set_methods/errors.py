# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""set methods: documented exception paths (CPython 3.12 oracle)."""


# pop on empty set raises KeyError.
try:
    set().pop()
    print("pop_empty: no_raise")
except KeyError as e:
    print("pop_empty:", type(e).__name__, str(e)[:60])


# remove of missing element raises KeyError (a LookupError subclass).
try:
    {1, 2}.remove(99)
    print("remove_missing: no_raise")
except LookupError as e:
    print("remove_missing:", type(e).__name__, isinstance(e, KeyError))


# discard of missing is silent (no raise).
{1, 2}.discard(99)
print("discard_silent: ok")


# Unhashable element raises TypeError.
try:
    {[1, 2]}  # type: ignore[misc]
    print("list_elem: no_raise")
except TypeError as e:
    print("list_elem:", type(e).__name__, str(e)[:60])


# __contains__ of an unhashable value raises TypeError.
try:
    [] in {1, 2}  # type: ignore[operator]
    print("contains_unhashable: no_raise")
except TypeError as e:
    print("contains_unhashable:", type(e).__name__)


# union with non-iterable raises TypeError.
try:
    {1}.union(42)  # type: ignore[arg-type]
    print("union_int: no_raise")
except TypeError as e:
    print("union_int:", type(e).__name__, str(e)[:60])


# difference / difference_update with a non-iterable raise TypeError (issue 37219).
try:
    set().difference(123)  # type: ignore[arg-type]
    print("difference_int: no_raise")
except TypeError as e:
    print("difference_int:", type(e).__name__)
try:
    set().difference_update(123)  # type: ignore[arg-type]
    print("difference_update_int: no_raise")
except TypeError as e:
    print("difference_update_int:", type(e).__name__)


# The & operator (unlike .intersection) refuses a plain iterable.
try:
    {1, 2} & [1, 2]  # type: ignore[operator]
    print("and_list: no_raise")
except TypeError as e:
    print("and_list:", type(e).__name__, str(e)[:60])


# Constructor rejects a second positional arg and keyword args.
try:
    set([], 2)  # type: ignore[call-arg]
    print("two_arg: no_raise")
except TypeError as e:
    print("two_arg:", type(e).__name__)
try:
    set().__init__(a=1)  # type: ignore[call-arg]
    print("init_kw: no_raise")
except TypeError as e:
    print("init_kw:", type(e).__name__)


# A set is unhashable; a frozenset is hashable.
try:
    hash({1, 2})  # type: ignore[arg-type]
    print("hash_set: no_raise")
except TypeError as e:
    print("hash_set:", type(e).__name__, str(e)[:60])
print("hash_frozenset:", isinstance(hash(frozenset([1, 2])), int))


# Mutating a frozenset raises AttributeError.
fs = frozenset([1, 2])
try:
    fs.add(3)  # type: ignore[attr-defined]
    print("frozen_add: no_raise")
except AttributeError as e:
    print("frozen_add:", type(e).__name__, str(e)[:60])


# Happy: union returns a new set.
print("union:", {1, 2} | {2, 3})
