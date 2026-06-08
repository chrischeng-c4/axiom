# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "behavior"
# case = "priority_order"
# subject = "queue.PriorityQueue"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_queue.py"
# status = "filled"
# ///
"""queue.PriorityQueue: PriorityQueue dequeues by ascending priority regardless of insertion order: (5,e),(1,a),(3,c),(2,b),(4,d) drain sorted by key"""
import queue

pq = queue.PriorityQueue()
pq.put((5, "e"))
pq.put((1, "a"))
pq.put((3, "c"))
pq.put((2, "b"))
pq.put((4, "d"))
drained = [pq.get() for _ in range(5)]
assert drained == [(1, "a"), (2, "b"), (3, "c"), (4, "d"), (5, "e")], \
    f"priority order = {drained!r}"

print("priority_order OK")
