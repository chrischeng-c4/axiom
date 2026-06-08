# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "behavior"
# case = "maxsize_zero_is_unlimited"
# subject = "queue.Queue"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_queue.py"
# status = "filled"
# ///
"""queue.Queue: maxsize=0 means unlimited capacity: 100 puts succeed, qsize()==100 and full() stays False"""
import queue

q = queue.Queue(maxsize=0)
for i in range(100):
    q.put(i)
assert q.qsize() == 100, f"unlimited qsize = {q.qsize()!r}"
assert q.full() is False, "unlimited queue never full"

print("maxsize_zero_is_unlimited OK")
