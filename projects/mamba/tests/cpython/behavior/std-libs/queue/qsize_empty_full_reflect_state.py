# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "behavior"
# case = "qsize_empty_full_reflect_state"
# subject = "queue.Queue"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_queue.py"
# status = "filled"
# ///
"""queue.Queue: qsize()/empty()/full() track current occupancy: a maxsize=3 queue reports empty then full as items are put, and not-full after a get"""
import queue

q = queue.Queue(maxsize=3)
assert q.qsize() == 0, "qsize 0 initially"
assert q.empty() is True, "empty initially"
assert q.full() is False, "not full initially"
q.put("a")
assert q.qsize() == 1, f"qsize 1: {q.qsize()!r}"
q.put("b")
q.put("c")
assert q.full() is True, f"full at max: {q.full()!r}"
assert q.empty() is False, "not empty when full"
q.get()
assert q.full() is False, "not full after get"

print("qsize_empty_full_reflect_state OK")
