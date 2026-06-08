# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "heapsort_property_both_paths"
# subject = "heapq.heapify"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heapify: draining a heap yields the same order as sorted() for both heap-construction paths (bulk heapify and incremental heappush), and nlargest/nsmallest agree with the sorted slices, over a deterministic pseudo-random sequence across several sizes"""
import heapq


def gen(n, seed=12345):
    """Deterministic LCG -> values in [0, 1000); no random module."""
    out = []
    state = seed
    for _ in range(n):
        state = (1103515245 * state + 12345) & 0x7FFFFFFF
        out.append(state % 1000)
    return out


for size in (0, 1, 2, 7, 50, 200):
    data = gen(size, seed=size + 1)
    expected = sorted(data)

    # Path A: bulk heapify, then drain.
    heap_a = data[:]
    heapq.heapify(heap_a)
    drained_a = [heapq.heappop(heap_a) for _ in range(size)]
    assert drained_a == expected, f"heapify-drain size={size}"
    assert heap_a == [], f"heap fully drained size={size}"

    # Path B: incremental heappush, then drain.
    heap_b = []
    for item in data:
        heapq.heappush(heap_b, item)
    drained_b = [heapq.heappop(heap_b) for _ in range(size)]
    assert drained_b == expected, f"heappush-drain size={size}"

    # nlargest/nsmallest agree with the sorted slices.
    for k in (0, 1, 3, size, size + 5):
        assert heapq.nlargest(k, data) == expected[::-1][:k], (
            f"nlargest k={k} size={size}"
        )
        assert heapq.nsmallest(k, data) == expected[:k], (
            f"nsmallest k={k} size={size}"
        )
print("heapsort_property_both_paths OK")
