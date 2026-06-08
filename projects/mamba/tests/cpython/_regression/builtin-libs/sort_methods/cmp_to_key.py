# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""sort methods: functools.cmp_to_key adapter and mutation detection.

cmp_to_key turns an old-style 3-way comparison function into a key function.
Sorting also guards against the underlying list being mutated mid-sort.
"""

from functools import cmp_to_key


def three_way(a, b):
    """Classic comparator: -1, 0, or 1."""
    return (a > b) - (a < b)


# A cmp_to_key comparator sorts identically to the natural order.
nums = [3, 1, 4, 1, 5, 9, 2, 6]
assert sorted(nums, key=cmp_to_key(three_way)) == sorted(nums)
print("cmp_basic:", sorted(nums, key=cmp_to_key(three_way)))


# cmp_to_key on a case-folding comparator matches the str.lower key.
words = "The quick Brown fox Jumped over The lazy Dog".split()


def cmp_lower(a, b):
    al, bl = a.lower(), b.lower()
    return (al > bl) - (al < bl)


via_cmp = sorted(words, key=cmp_to_key(cmp_lower))
via_key = sorted(words, key=str.lower)
assert via_cmp == via_key
print("cmp_eq_keylower: ok")


# Reversing the comparator's arguments yields a descending sort, which
# equals key+reverse=True.
def cmp_lower_rev(a, b):
    al, bl = a.lower(), b.lower()
    return (bl > al) - (bl < al)


assert sorted(words, key=cmp_to_key(cmp_lower_rev)) == sorted(
    words, key=str.lower, reverse=True
)
print("cmp_reversed_eq: ok")


# Mutating the list from inside a cmp_to_key comparator is detected and
# raises ValueError ("list modified during sort").
target = [1, 2, 3, 4, 5]


def mutating(a, b):
    target.append(0)
    target.pop()
    return (a > b) - (a < b)


try:
    target.sort(key=cmp_to_key(mutating))
    print("mutate_cmp: no_raise")
except ValueError as e:
    print("mutate_cmp: ValueError", str(e)[:40])


# Same guard fires when a plain key function mutates the list.
plain = list(range(10))


def mutate_key(x):
    plain[:] = range(20)
    return x


try:
    plain.sort(key=mutate_key)
    print("mutate_key: no_raise")
except ValueError as e:
    print("mutate_key: ValueError", str(e)[:40])

print("cmp_to_key OK")
