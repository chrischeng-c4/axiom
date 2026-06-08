# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "atomicity"
# lib = "set"
# dimension = "concurrency"
# case = "add_distinct_no_corruption"
# subject = "set.add"
# kind = "semantic"
# mem_carveout = ""
# source = "concurrency_matrix.py (no-corruption absolute)"
# status = "filled"
# ///
"""Concurrency absolute: a shared set must never be CORRUPTED by concurrent add.

N threads add K *distinct* keys each — `(tid, i)` for tid in 0..N-1, i in 0..K-1
— so all N*K keys are unique. Whatever the scheduling, the final set must contain
exactly N*K elements (a single set.add is atomic under the contract; even a
fully-serialized runtime must accumulate all distinct keys). This previously
collapsed to one thread's worth because the Thread `args` were never delivered
(every worker saw the same garbage `tid`); fixed once `start()` passes
`target(*args, **kwargs)`.
"""
import threading

N, K = 4, 1000
shared = set()


def worker(tid: int) -> None:
    for i in range(K):
        shared.add((tid, i))


threads = [threading.Thread(target=worker, args=(t,)) for t in range(N)]
for t in threads:
    t.start()
for t in threads:
    t.join()

got = len(shared)
if got == N * K:
    print("concurrency: PASS")
else:
    print(f"concurrency: FAIL: corrupted set, len={got} expected={N * K}")
