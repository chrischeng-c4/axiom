# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "safety"
# lib = "lock"
# dimension = "concurrency"
# case = "counter_under_lock_is_exact"
# subject = "threading.Lock"
# kind = "semantic"
# mem_carveout = ""
# source = "concurrency_matrix.py (caller-locked compound op)"
# status = "filled"
# ///
"""Concurrency contract: a COMPOUND op is the caller's job to lock — and a Lock
makes it exact.

`counter[0] += 1` is NOT atomic (load-add-store); under the contract it may race
and lose updates when run truly in parallel without synchronization (the caller
must lock — mamba does not over-promise compound atomicity). This fixture asserts
the CORRECT pattern: guarding the compound op with threading.Lock yields the
exact total N*K every time. This is the property a real lock must uphold once the
runtime is genuinely parallel.
"""
import threading

N, K = 4, 1000
counter = [0]
lock = threading.Lock()


def worker(_tid: int) -> None:
    for _ in range(K):
        with lock:
            counter[0] += 1


threads = [threading.Thread(target=worker, args=(t,)) for t in range(N)]
for t in threads:
    t.start()
for t in threads:
    t.join()

got = counter[0]
if got == N * K:
    print("concurrency: PASS")
else:
    print(f"concurrency: FAIL: lock did not serialize, total={got} expected={N * K}")
