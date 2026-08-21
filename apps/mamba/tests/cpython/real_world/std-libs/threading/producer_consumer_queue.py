# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "real_world"
# case = "producer_consumer_queue"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: a bounded producer/consumer pipeline: a producer thread enqueues N items into a queue.Queue and a consumer thread dequeues and sums them, coordinated only by the queue, with the total verified after join"""
import threading

import queue

N = 500
q = queue.Queue(maxsize=16)
SENTINEL = object()
consumed = []


def producer():
    for i in range(N):
        q.put(i)
    q.put(SENTINEL)


def consumer():
    while True:
        item = q.get()
        if item is SENTINEL:
            q.task_done()
            break
        consumed.append(item)
        q.task_done()


prod = threading.Thread(target=producer, name="producer")
cons = threading.Thread(target=consumer, name="consumer")
prod.start()
cons.start()
prod.join()
cons.join()

# The pipeline transports every item exactly once, in order, with the queue as
# the only coordination primitive.
assert len(consumed) == N, f"consumed count = {len(consumed)!r}"
assert consumed == list(range(N)), "items arrived out of order or duplicated"
assert sum(consumed) == N * (N - 1) // 2, f"sum = {sum(consumed)!r}"
print("produced_consumed:", len(consumed))
print("checksum:", sum(consumed))

print("producer_consumer_queue OK")
