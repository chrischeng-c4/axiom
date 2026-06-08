# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "max_workers_caps_concurrency"
# subject = "concurrent.futures.ThreadPoolExecutor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.ThreadPoolExecutor: max_workers=2 caps simultaneously-running tasks at 2; a concurrency tracker across six tasks never observes more than two active at once"""
import concurrent.futures
import threading
import time

active = [0]
max_active = [0]
lock = threading.Lock()


def track_concurrency():
    with lock:
        active[0] += 1
        if active[0] > max_active[0]:
            max_active[0] = active[0]
    time.sleep(0.05)
    with lock:
        active[0] -= 1


with concurrent.futures.ThreadPoolExecutor(max_workers=2) as ex:
    futs = [ex.submit(track_concurrency) for _ in range(6)]
    for f in futs:
        f.result(timeout=5)

assert max_active[0] <= 2, f"max_workers=2 cap respected: peak active = {max_active[0]!r}"
assert max_active[0] >= 1, "at least one task observed running"

print("max_workers_caps_concurrency OK")
