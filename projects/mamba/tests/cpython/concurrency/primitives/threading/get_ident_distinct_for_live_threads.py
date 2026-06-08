# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "primitives"
# lib = "threading"
# dimension = "concurrency"
# case = "get_ident_distinct_for_live_threads"
# subject = "threading.get_ident"
# kind = "semantic"
# mem_carveout = ""
# source = "concurrency_matrix.py (thread identity primitive)"
# status = "filled"
# ///
"""Concurrency primitive: simultaneously-live threads have distinct identities.

A Barrier(N) holds all N threads alive at the same instant (so the OS cannot
recycle a thread id), then each records threading.get_ident(). The contract:
N concurrently-live threads yield N distinct ids. CPython (GIL and free-threaded
alike) gives exactly N. mamba returns one shared id for all threads, so thread
identity is currently unusable for keying thread-local state.
"""
import threading

N = 6
ids: list[int] = []
collect = threading.Lock()
gate = threading.Barrier(N)


def worker() -> None:
    gate.wait()  # all N rendezvous → all alive together
    me = threading.get_ident()
    with collect:
        ids.append(me)


threads = [threading.Thread(target=worker) for _ in range(N)]
for t in threads:
    t.start()
for t in threads:
    t.join()

distinct = len(set(ids))
if distinct == N:
    print("concurrency: PASS")
else:
    print(f"concurrency: FAIL: {distinct} distinct ids for {N} live threads")
