# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "atomicity"
# lib = "list"
# dimension = "concurrency"
# case = "append_no_lost_updates"
# subject = "list.append"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "concurrency_matrix.py (single-mutation atomicity contract)"
# status = "filled"
# ///
"""Concurrency contract: a SINGLE list.append is atomic.

N threads each append K items to one shared list with no explicit lock. Under
the contract (= free-threaded CPython 3.13t, which takes a per-object critical
section on the append) every append lands: len == N*K, no lost updates, no
corruption. A truly-parallel runtime that did NOT lock the append would lose
items here. Verdict is deterministic: the property either holds or it does not.
"""
import threading

N, K = 4, 1000
shared: list[int] = []


def worker(tid: int) -> None:
    for _ in range(K):
        shared.append(tid)


threads = [threading.Thread(target=worker, args=(t,)) for t in range(N)]
for t in threads:
    t.start()
for t in threads:
    t.join()

got = len(shared)
if got == N * K:
    print("concurrency: PASS")
else:
    print(f"concurrency: FAIL: lost updates, len={got} expected={N * K}")
