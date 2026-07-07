"""go_suite shape: queue producer/consumer.

Server-shaped: a bounded work queue where producers enqueue typed tasks and
workers dequeue + process them -- the shape of an in-process job queue or
request-fan-out worker pool. Uses `queue.Queue` (FIFO) for the actual
enqueue/dequeue data-structure traffic.

Deliberately single-threaded/deterministic: real OS-thread interleaving would
make the checksum path-order-dependent (and mamba's threading-throughput
story is explicitly out of this epic's scope -- see epic #1071 "Concurrency
competitiveness ... is out of this epic's throughput scope"). The producer/
consumer roles are simulated by round-robin draining a bounded queue, which
still exercises the real `queue.Queue` put/get hot path this shape is meant
to measure, without adding OS-thread nondeterminism to the correctness gate.
"""

import queue


class Task:
    def __init__(self, task_id: int, priority: int, payload: int) -> None:
        self.task_id: int = task_id
        self.priority: int = priority
        self.payload: int = payload


def process_task(t: Task) -> int:
    # deterministic "work": a small mix so results depend on all 3 fields
    return (t.payload * 31 + t.priority * 7 + t.task_id) % 1000003


def checksum(data: bytes) -> int:
    h: int = 0
    mod: int = 1000000007
    mult: int = 131
    for b in data:
        h = (h * mult + b) % mod
    return h


def main() -> None:
    n_tasks = 4000
    max_inflight = 64  # bounded queue: producer stalls if consumer falls behind
    q: queue.Queue = queue.Queue(max_inflight)

    produced = 0
    consumed = 0
    results: list[int] = []

    while consumed < n_tasks:
        # producer: fill up to max_inflight or until all tasks are produced
        while produced < n_tasks and not q.full():
            t = Task(produced, produced % 5, (produced * 17) % 10007)
            q.put(t)
            produced += 1
        # consumer: drain a batch before producing more (round-robin shape)
        drained_this_round = 0
        while not q.empty() and drained_this_round < 32:
            t = q.get()
            results.append(process_task(t))
            consumed += 1
            drained_this_round += 1

    total = 0
    for r in results:
        total = (total + r) % 1000000007
    summary = "queue_pipeline:" + str(produced) + ":" + str(consumed) + ":" + str(total)
    print("CHECKSUM", checksum(summary.encode("utf-8")))


main()
