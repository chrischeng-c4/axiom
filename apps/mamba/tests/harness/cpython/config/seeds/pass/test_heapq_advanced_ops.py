# Operational AssertionPass seed for heapq surfaces beyond
# test_heapq_ops (which covers heapify, heappush, heappop,
# nsmallest, nlargest).
# Surface: heappushpop combines push+pop atomically; heapreplace
# also combines pop+push atomically (push first); merge over two
# pre-sorted iterables; merge over three pre-sorted iterables;
# heap-sort via repeated heappop produces a sorted sequence; manual
# heappush of mixed values preserves the min-heap invariant.
import heapq
_ledger: list[int] = []

# heappushpop pushes the item, then pops the smallest. The popped
# value is the smaller of (new_item, old_smallest).
hh = [1, 3, 5]
heapq.heapify(hh)
v = heapq.heappushpop(hh, 4)
# 1 was already smallest; pushpop returns 1 and leaves [3, 4, 5] (any order
# that preserves the heap invariant)
assert v == 1; _ledger.append(1)
assert sorted(hh) == [3, 4, 5]; _ledger.append(1)

# heapreplace pops the smallest first, then pushes the new item — even
# when the new item is larger than everything else in the heap
hh2 = [1, 3, 5]
heapq.heapify(hh2)
v2 = heapq.heapreplace(hh2, 10)
assert v2 == 1; _ledger.append(1)
assert sorted(hh2) == [3, 5, 10]; _ledger.append(1)

# merge over two pre-sorted iterables yields the fully sorted sequence
assert list(heapq.merge([1, 3, 5], [2, 4, 6])) == [1, 2, 3, 4, 5, 6]; _ledger.append(1)
# merge handles three iterables
assert list(heapq.merge([1, 4, 7], [2, 5, 8], [3, 6, 9])) == [1, 2, 3, 4, 5, 6, 7, 8, 9]; _ledger.append(1)
# merge over one iterable just passes through
assert list(heapq.merge([1, 2, 3])) == [1, 2, 3]; _ledger.append(1)
# merge with an empty iterable returns the other
assert list(heapq.merge([], [1, 2, 3])) == [1, 2, 3]; _ledger.append(1)

# Heap sort: repeated heappop produces values in ascending order
src = [5, 3, 8, 1, 9, 2]
heapq.heapify(src)
sorted_out = []
while src:
    sorted_out.append(heapq.heappop(src))
assert sorted_out == [1, 2, 3, 5, 8, 9]; _ledger.append(1)

# Manual heappush of mixed values: smallest is always at index 0
h: list[int] = []
for v3 in [5, 3, 8, 1, 9, 2]:
    heapq.heappush(h, v3)
# After all pushes, the smallest sits at the root
assert h[0] == 1; _ledger.append(1)
# Popping yields the smallest, then the next smallest
assert heapq.heappop(h) == 1; _ledger.append(1)
assert heapq.heappop(h) == 2; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_heapq_advanced_ops {sum(_ledger)} asserts")
