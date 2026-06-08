# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "thread_pool_runs_all_tasks"
# subject = "concurrent.futures.ThreadPoolExecutor.submit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.ThreadPoolExecutor.submit: every submitted task runs to completion under a 4-worker pool: submitting i->i*i for range(10) records all ten inputs and returns all ten squares"""
import concurrent.futures
import threading

seen = []
lock = threading.Lock()


def work(n):
    with lock:
        seen.append(n)
    return n * n


with concurrent.futures.ThreadPoolExecutor(max_workers=4) as ex:
    futs = [ex.submit(work, i) for i in range(10)]
    squares = [f.result(timeout=5) for f in futs]

assert sorted(seen) == list(range(10)), f"all ten tasks ran: {sorted(seen)!r}"
assert sorted(squares) == [i * i for i in range(10)], f"squares: {sorted(squares)!r}"

print("thread_pool_runs_all_tasks OK")
