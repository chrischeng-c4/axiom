# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_list_methods"
# subject = "cpython321.lang_list_methods"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_list_methods.py"
# status = "filled"
# ///
"""cpython321.lang_list_methods: execute CPython 3.12 seed lang_list_methods"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for list method surfaces.
# Surface: append (single-element add), extend (in-place concat),
# pop (default last, by index), index (first-occurrence search),
# insert (positional add), remove (first-occurrence delete), clear
# (empty in place), sort (in place, with reverse= and key=),
# reverse (in place), count (occurrence tally), copy (independent
# top-level), slice assignment (lst[a:b] = ...), concatenation (+
# operator), repetition (* operator), membership (in / not in),
# len / min / max / sum / sorted / reversed (top-level helpers),
# nested list subscript access, list() from str / range iterables.
_ledger: list[int] = []

# append — add one element at the end
a = [1, 2]
a.append(3)
assert a == [1, 2, 3]; _ledger.append(1)
a.append(4)
assert a == [1, 2, 3, 4]; _ledger.append(1)

# extend — in-place concatenation
b = [1, 2]
b.extend([3, 4])
assert b == [1, 2, 3, 4]; _ledger.append(1)
b.extend([])
assert b == [1, 2, 3, 4]; _ledger.append(1)

# pop with no index removes the LAST element
c = [1, 2, 3]
last = c.pop()
assert last == 3; _ledger.append(1)
assert c == [1, 2]; _ledger.append(1)
# pop with an explicit index removes that one
d = [10, 20, 30]
first = d.pop(0)
assert first == 10; _ledger.append(1)
assert d == [20, 30]; _ledger.append(1)

# index — first occurrence position
assert [10, 20, 30].index(20) == 1; _ledger.append(1)
assert [1, 2, 1, 2].index(2) == 1; _ledger.append(1)

# insert — positional add
e = [1, 3]
e.insert(1, 2)
assert e == [1, 2, 3]; _ledger.append(1)
e.insert(0, 0)
assert e == [0, 1, 2, 3]; _ledger.append(1)
# insert past the end appends
e.insert(100, 99)
assert e == [0, 1, 2, 3, 99]; _ledger.append(1)

# remove — only the first matching occurrence
f = [1, 2, 3, 2]
f.remove(2)
assert f == [1, 3, 2]; _ledger.append(1)

# clear — empty the list in place
g = [1, 2, 3]
g.clear()
assert g == []; _ledger.append(1)
assert len(g) == 0; _ledger.append(1)

# sort — in place, ascending by default
h = [3, 1, 2]
h.sort()
assert h == [1, 2, 3]; _ledger.append(1)
# sort with reverse=True — descending
i = [3, 1, 2]
i.sort(reverse=True)
assert i == [3, 2, 1]; _ledger.append(1)
# sort with key= — by computed value (here, str length)
j = ["bbb", "a", "cc"]
j.sort(key=len)
assert j == ["a", "cc", "bbb"]; _ledger.append(1)

# reverse — in place, no return value used
k = [1, 2, 3]
k.reverse()
assert k == [3, 2, 1]; _ledger.append(1)

# count — number of equal occurrences
assert [1, 2, 2, 3].count(2) == 2; _ledger.append(1)
assert [1, 2, 3].count(99) == 0; _ledger.append(1)
assert [1, 1, 1, 1].count(1) == 4; _ledger.append(1)

# copy — top-level independent
m = [1, 2, 3]
n = m.copy()
n.append(4)
assert m == [1, 2, 3]; _ledger.append(1)
assert n == [1, 2, 3, 4]; _ledger.append(1)

# Slice assignment — replace a slice with another iterable
p = [1, 2, 3, 4, 5]
p[1:3] = [9, 9]
assert p == [1, 9, 9, 4, 5]; _ledger.append(1)
# Slice assign with a different-length RHS resizes the list
q = [1, 2, 3, 4, 5]
q[1:4] = [99]
assert q == [1, 99, 5]; _ledger.append(1)

# Concatenation and repetition build new lists
assert [1, 2] + [3, 4] == [1, 2, 3, 4]; _ledger.append(1)
assert [1, 2] * 3 == [1, 2, 1, 2, 1, 2]; _ledger.append(1)
assert [1] * 5 == [1, 1, 1, 1, 1]; _ledger.append(1)
assert [] * 5 == []; _ledger.append(1)

# Membership
assert 2 in [1, 2, 3]; _ledger.append(1)
assert 5 not in [1, 2, 3]; _ledger.append(1)

# Top-level helpers — len, min, max, sum, sorted, reversed
r = [3, 1, 2]
assert len(r) == 3; _ledger.append(1)
assert min(r) == 1; _ledger.append(1)
assert max(r) == 3; _ledger.append(1)
assert sum([1, 2, 3]) == 6; _ledger.append(1)
assert sorted(r) == [1, 2, 3]; _ledger.append(1)
assert list(reversed([1, 2, 3])) == [3, 2, 1]; _ledger.append(1)

# Nested subscript access
m2 = [[1, 2], [3, 4], [5, 6]]
assert m2[0][1] == 2; _ledger.append(1)
assert m2[2][0] == 5; _ledger.append(1)
assert m2[-1][-1] == 6; _ledger.append(1)

# list() from str / range / tuple iterables
assert list("abc") == ["a", "b", "c"]; _ledger.append(1)
assert list(range(3)) == [0, 1, 2]; _ledger.append(1)
assert list((1, 2, 3)) == [1, 2, 3]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_list_methods {sum(_ledger)} asserts")
