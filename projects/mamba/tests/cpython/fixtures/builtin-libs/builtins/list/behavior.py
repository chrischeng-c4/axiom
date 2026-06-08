"""Behavior contract for builtins.list.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: list() with no args returns []
assert list() == [], f"list() = {list()!r}"

# Rule 2: list(iterable) — from range, tuple, str, set
assert list(range(3)) == [0, 1, 2], f"list(range(3)) = {list(range(3))!r}"
assert list((1, 2)) == [1, 2], f"list((1,2)) = {list((1,2))!r}"
assert list("abc") == ["a", "b", "c"], f"list('abc') = {list('abc')!r}"

# Rule 3: append
lst = [1, 2]
lst.append(3)
assert lst == [1, 2, 3], f"append = {lst!r}"

# Rule 4: extend
lst = [1, 2]
lst.extend([3, 4])
assert lst == [1, 2, 3, 4], f"extend = {lst!r}"

# Rule 5: insert
lst = [1, 3]
lst.insert(1, 2)
assert lst == [1, 2, 3], f"insert = {lst!r}"

# Rule 6: remove (first occurrence)
lst = [1, 2, 1, 3]
lst.remove(1)
assert lst == [2, 1, 3], f"remove = {lst!r}"

# Rule 7: remove raises ValueError for missing element
_raised = False
try:
    [1, 2].remove(99)
except ValueError:
    _raised = True
assert _raised, "remove(99) did not raise ValueError"

# Rule 8: pop — default last, index-based
lst = [1, 2, 3]
assert lst.pop() == 3, f"pop() = {lst.pop()!r}"
lst = [1, 2, 3]
assert lst.pop(0) == 1, f"pop(0) = {lst.pop(0)!r}"
assert lst == [2, 3], f"after pop(0): {lst!r}"

# Rule 9: index / count
lst = [1, 2, 3, 2]
assert lst.index(2) == 1, f"index = {lst.index(2)!r}"
assert lst.count(2) == 2, f"count = {lst.count(2)!r}"

# Rule 10: sort (in-place, stable)
lst = [3, 1, 4, 1, 5, 9, 2, 6]
lst.sort()
assert lst == [1, 1, 2, 3, 4, 5, 6, 9], f"sort = {lst!r}"
lst = ["banana", "apple", "cherry"]
lst.sort()
assert lst == ["apple", "banana", "cherry"], f"sort str = {lst!r}"

# Rule 11: sort reverse=True
lst = [3, 1, 2]
lst.sort(reverse=True)
assert lst == [3, 2, 1], f"sort(reverse=True) = {lst!r}"

# Rule 12: reverse (in-place)
lst = [1, 2, 3]
lst.reverse()
assert lst == [3, 2, 1], f"reverse = {lst!r}"

# Rule 13: slicing
lst = [0, 1, 2, 3, 4]
assert lst[1:3] == [1, 2], f"lst[1:3] = {lst[1:3]!r}"
assert lst[::2] == [0, 2, 4], f"lst[::2] = {lst[::2]!r}"
assert lst[::-1] == [4, 3, 2, 1, 0], f"lst[::-1] = {lst[::-1]!r}"

# Rule 14: concatenation / repetition
assert [1, 2] + [3, 4] == [1, 2, 3, 4], "concatenation failed"
assert [0] * 3 == [0, 0, 0], "repetition failed"

# Rule 15: clear
lst = [1, 2, 3]
lst.clear()
assert lst == [], f"clear = {lst!r}"

# Rule 16: copy is shallow
lst = [1, [2, 3]]
cp = lst.copy()
assert cp == [1, [2, 3]], f"copy = {cp!r}"
cp[1].append(4)
assert lst[1] == [2, 3, 4], "copy is shallow: inner list shared"

# Rule 17: list comprehension
assert [x * 2 for x in range(4)] == [0, 2, 4, 6], "list comprehension failed"

print("behavior OK")
