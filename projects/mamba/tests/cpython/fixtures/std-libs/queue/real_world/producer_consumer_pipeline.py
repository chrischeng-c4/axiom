# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "real_world"
# case = "producer_consumer_pipeline"
# subject = "queue.Queue"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_queue.py"
# status = "filled"
# ///
"""queue.Queue: a producer enqueues a bounded work batch and a daemon consumer thread drains it via get/task_done; the producer join()s the queue and asserts every job was processed exactly once"""
import queue
import threading

JOBS = 50

work = queue.Queue()
processed = []
lock = threading.Lock()


def consumer():
    while True:
        job = work.get()
        # Realistic per-job work: square the job id.
        with lock:
            processed.append(job * job)
        work.task_done()


t = threading.Thread(target=consumer, daemon=True)
t.start()

# Producer side: enqueue a bounded batch of jobs.
for job in range(JOBS):
    work.put(job)

# Block until the consumer has marked every job done.
work.join()

assert len(processed) == JOBS, f"all jobs processed = {len(processed)!r}"
assert sorted(processed) == [n * n for n in range(JOBS)], \
    "each job processed exactly once with the expected result"

print("producer_consumer_pipeline OK")
