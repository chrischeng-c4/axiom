# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "atomicity"
# lib = "dict"
# dimension = "concurrency"
# case = "distinct_key_setitem_no_loss"
# subject = "dict.__setitem__"
# kind = "semantic"
# mem_carveout = ""
# source = "concurrency_matrix.py (single-mutation atomicity contract)"
# status = "filled"
# ///
"""Concurrency contract: a SINGLE dict[k] = v is atomic; distinct keys never lost.

N threads each write K disjoint keys (`tid * K + i`) into one shared dict with no
lock. Keys never collide across threads, so the final dict must hold exactly N*K
entries — a single __setitem__ is atomic under the contract, and even a
serialized runtime accumulates all distinct keys. A runtime whose concurrent
insert corrupts the table's buckets would lose keys here.
"""
import threading

N, K = 4, 1000
shared: dict[int, int] = {}


def worker(tid: int) -> None:
    base = tid * K
    for i in range(K):
        shared[base + i] = tid


threads = [threading.Thread(target=worker, args=(t,)) for t in range(N)]
for t in threads:
    t.start()
for t in threads:
    t.join()

got = len(shared)
if got == N * K:
    print("concurrency: PASS")
else:
    print(f"concurrency: FAIL: lost keys, len={got} expected={N * K}")
