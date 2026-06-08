# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "behavior"
# case = "fifo_order"
# subject = "queue.Queue"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_queue.py"
# status = "filled"
# ///
"""queue.Queue: Queue is FIFO: putting 10,20,30,40,50 then draining gives them back in insertion order"""
import queue

q = queue.Queue()
for item in [10, 20, 30, 40, 50]:
    q.put(item)
drained = [q.get() for _ in range(5)]
assert drained == [10, 20, 30, 40, 50], f"FIFO order = {drained!r}"

print("fifo_order OK")
