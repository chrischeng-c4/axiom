# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "behavior"
# case = "lifo_order"
# subject = "queue.LifoQueue"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_queue.py"
# status = "filled"
# ///
"""queue.LifoQueue: LifoQueue is LIFO (stack): putting 1,2,3,4 then draining gives 4,3,2,1"""
import queue

lq = queue.LifoQueue()
for item in [1, 2, 3, 4]:
    lq.put(item)
drained = [lq.get() for _ in range(4)]
assert drained == [4, 3, 2, 1], f"LIFO order = {drained!r}"

print("lifo_order OK")
