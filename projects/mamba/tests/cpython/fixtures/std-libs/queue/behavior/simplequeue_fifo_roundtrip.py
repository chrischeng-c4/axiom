# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "behavior"
# case = "simplequeue_fifo_roundtrip"
# subject = "queue.SimpleQueue"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_queue.py"
# status = "filled"
# ///
"""queue.SimpleQueue: SimpleQueue is an unbounded FIFO: put 100 then 200 and get them back in order"""
import queue

sq = queue.SimpleQueue()
sq.put(100)
sq.put(200)
assert sq.get() == 100, "SimpleQueue FIFO first"
assert sq.get() == 200, "SimpleQueue FIFO second"
assert sq.empty() is True, "SimpleQueue empty after drain"

print("simplequeue_fifo_roundtrip OK")
