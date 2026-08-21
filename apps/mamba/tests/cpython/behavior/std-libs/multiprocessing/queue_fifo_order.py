# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "behavior"
# case = "queue_fifo_order"
# subject = "multiprocessing.Queue"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/_test_multiprocessing.py"
# status = "filled"
# ///
"""multiprocessing.Queue: a Queue is FIFO and process-safe; putting 0..4 then getting five times returns [0,1,2,3,4] in order (spawn-guarded under __main__)"""
import multiprocessing

if __name__ == "__main__":
    q = multiprocessing.Queue()
    for i in range(5):
        q.put(i)
    got = []
    for _ in range(5):
        got.append(q.get(timeout=5))
    assert got == [0, 1, 2, 3, 4], f"Queue FIFO: {got!r}"

    print("queue_fifo_order OK")
