# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "behavior"
# case = "task_done_join_unblocks"
# subject = "queue.Queue.join"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_queue.py"
# status = "filled"
# ///
"""queue.Queue.join: join() blocks until every queued item is task_done(): a worker thread drains 3 items and the main thread's join() returns once all are marked done"""
import queue
import threading

q = queue.Queue()
results = []
q.put("work1")
q.put("work2")
q.put("work3")


def worker():
    while True:
        item = q.get()
        results.append(item)
        q.task_done()


t = threading.Thread(target=worker, daemon=True)
t.start()
q.join()  # blocks until every task_done() has been called
assert sorted(results) == ["work1", "work2", "work3"], \
    f"worker results = {results!r}"

print("task_done_join_unblocks OK")
