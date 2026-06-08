# Operational AssertionPass seed for heapq surfaces beyond test_heapq_ops
# and test_heapq_advanced_ops. Focus: combining/mutating operators —
# heappushpop (push then pop in one step, returns the smaller of the
# new element vs. the existing min); heapreplace (pop then push,
# returns the original min and inserts the new element); merge
# (heap-merges any number of pre-sorted iterables into one sorted
# stream); heapify-then-pop-all yields a sorted list (heapsort);
# nlargest / nsmallest without a key=, including the n > len case.
import heapq
_ledger: list[int] = []

# heappushpop — when the new element is LARGER than the current min,
# the current min is popped and returned (new element ends up in heap)
a = [1, 3, 5]
heapq.heapify(a)
assert heapq.heappushpop(a, 4) == 1; _ledger.append(1)

# heappushpop — when the new element is SMALLER than the current min,
# the new element itself is returned (heap stays the same shape)
b = [2, 4, 6]
heapq.heapify(b)
assert heapq.heappushpop(b, 1) == 1; _ledger.append(1)
# The original min is still present (not popped, because 1 < 2)
assert 2 in b; _ledger.append(1)

# heapreplace — always pops the current min and pushes the new one,
# regardless of which is smaller; returns the popped min
c = [1, 3, 5]
heapq.heapify(c)
assert heapq.heapreplace(c, 4) == 1; _ledger.append(1)
# The new element is now in the heap (and 1 is gone)
assert 1 not in c; _ledger.append(1)
assert 4 in c; _ledger.append(1)

# heapreplace with a smaller-than-min replacement still pops the
# original min (this is what differentiates it from heappushpop)
d = [3, 5, 7]
heapq.heapify(d)
assert heapq.heapreplace(d, 1) == 3; _ledger.append(1)
assert 3 not in d; _ledger.append(1)
assert 1 in d; _ledger.append(1)

# merge — two pre-sorted iterables → one ascending stream
assert list(heapq.merge([1, 3, 5], [2, 4, 6])) == [1, 2, 3, 4, 5, 6]; _ledger.append(1)

# merge — three or more iterables
assert list(heapq.merge([1, 10], [2, 9], [3, 8])) == [1, 2, 3, 8, 9, 10]; _ledger.append(1)

# merge — one empty iterable degenerates to the other
assert list(heapq.merge([], [1, 2, 3])) == [1, 2, 3]; _ledger.append(1)
assert list(heapq.merge([1, 2, 3], [])) == [1, 2, 3]; _ledger.append(1)
# All empty → empty
assert list(heapq.merge([], [])) == []; _ledger.append(1)

# merge — equal elements from different iterables both appear (stable
# in the sense of preserving multiplicity)
assert list(heapq.merge([1, 2], [1, 2])) == [1, 1, 2, 2]; _ledger.append(1)

# merge — single-iterable case is just a copy
assert list(heapq.merge([1, 2, 3])) == [1, 2, 3]; _ledger.append(1)

# Heapsort — heapify, then pop-all yields ascending order
e = [9, 3, 7, 1, 5, 2, 8, 4, 6]
heapq.heapify(e)
out: list[int] = []
while e:
    out.append(heapq.heappop(e))
assert out == [1, 2, 3, 4, 5, 6, 7, 8, 9]; _ledger.append(1)

# Heapsort with duplicates
f = [3, 1, 4, 1, 5, 9, 2, 6, 5]
heapq.heapify(f)
out2: list[int] = []
while f:
    out2.append(heapq.heappop(f))
assert out2 == [1, 1, 2, 3, 4, 5, 5, 6, 9]; _ledger.append(1)

# nlargest / nsmallest (no key=)
assert heapq.nlargest(3, [1, 5, 2, 8, 3, 9, 4]) == [9, 8, 5]; _ledger.append(1)
assert heapq.nsmallest(3, [1, 5, 2, 8, 3, 9, 4]) == [1, 2, 3]; _ledger.append(1)
# n == 1 — degenerates to max/min in a list
assert heapq.nlargest(1, [3, 1, 2]) == [3]; _ledger.append(1)
assert heapq.nsmallest(1, [3, 1, 2]) == [1]; _ledger.append(1)
# n >= len — returns the whole input sorted appropriately
assert heapq.nlargest(10, [3, 1, 2]) == [3, 2, 1]; _ledger.append(1)
assert heapq.nsmallest(10, [3, 1, 2]) == [1, 2, 3]; _ledger.append(1)

# Single-element heap edge case
g: list[int] = []
heapq.heappush(g, 42)
assert heapq.heappop(g) == 42; _ledger.append(1)
assert g == []; _ledger.append(1)

# A round-trip of push-then-pushpop-then-pop-all yields a sorted list
h: list[int] = []
heapq.heappush(h, 5)
heapq.heappush(h, 1)
heapq.heappush(h, 3)
assert heapq.heappushpop(h, 2) == 1; _ledger.append(1)
out3: list[int] = []
while h:
    out3.append(heapq.heappop(h))
assert out3 == [2, 3, 5]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_heapq_merge_ops {sum(_ledger)} asserts")
