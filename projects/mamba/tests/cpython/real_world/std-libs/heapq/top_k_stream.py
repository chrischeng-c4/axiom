# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "real_world"
# case = "top_k_stream"
# subject = "heapq.heappushpop"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heappushpop: streaming top-K log pipeline: offline nlargest/nsmallest batch top-K, a running size-K min-heap via heappushpop, a one-shot heapreplace root swap, and merge of two sorted runs -- the streaming result must equal the offline batch"""
import heapq

# A small synthetic event stream: each entry is a "priority score" that a real
# pipeline would derive from rate-limiting / fraud-detection logic.
STREAM_A = [5, 27, 11, 3, 41, 17, 8, 29, 13, 37, 2, 47, 19, 31, 7]
STREAM_B = [10, 50, 4, 22, 38, 15, 44, 6, 28, 33]
K = 5

# --- 1. Offline batch top-K via nlargest (the simplest heapq idiom). ---
top_k_batch = heapq.nlargest(K, STREAM_A)
print("offline top-5 of STREAM_A:", top_k_batch)
assert top_k_batch[0] == 47, f"top-K[0] should be the max, got {top_k_batch[0]}"
assert len(top_k_batch) == K

# --- 2. Offline batch bottom-K via nsmallest (mirror op). ---
bot_k_batch = heapq.nsmallest(K, STREAM_A)
print("offline bottom-5 of STREAM_A:", bot_k_batch)
assert bot_k_batch[0] == 2, f"bottom-K[0] should be the min, got {bot_k_batch[0]}"
assert len(bot_k_batch) == K

# --- 3. Streaming running top-K with a size-K min-heap.
# Keep the smallest of the current top-K at the root, and replace whenever a
# bigger value arrives. O(log K) per record vs O(N log N) for re-sorting.
running = []
heapq.heapify(running)
for v in STREAM_A:
    if len(running) < K:
        heapq.heappush(running, v)
    else:
        # heappushpop returns the smaller of (root, v); if v is larger, the
        # root has been replaced and we keep the same K count.
        heapq.heappushpop(running, v)

# Sort the K-heap descending to get the top-K answer.
streaming_top_k = sorted(running, reverse=True)
print("streaming top-5 of STREAM_A:", streaming_top_k)
assert streaming_top_k == top_k_batch, (
    f"streaming result diverged from offline: {streaming_top_k} != {top_k_batch}"
)

# --- 4. heapreplace as a one-shot root-swap during a merge phase.
merge_heap = list(STREAM_B)
heapq.heapify(merge_heap)
displaced = heapq.heapreplace(merge_heap, 100)
print("heapreplace displaced root:", displaced)
assert displaced == 4, f"heapreplace should return prior min, got {displaced}"

# --- 5. merge two sorted runs (offline use case). The output of sorted() is
# itself a sorted run; merge combines them into a single sorted result.
run_a = sorted(STREAM_A)
run_b = sorted(STREAM_B)
combined = list(heapq.merge(run_a, run_b))
print("merged length:", len(combined))
assert len(combined) == len(STREAM_A) + len(STREAM_B)
# Verify the merge is sorted ascending.
for i in range(1, len(combined)):
    assert combined[i - 1] <= combined[i], (
        f"merge not sorted at index {i}: {combined[i-1]} > {combined[i]}"
    )

print("top_k_stream OK")
