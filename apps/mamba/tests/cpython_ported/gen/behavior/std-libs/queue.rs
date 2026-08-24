use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/queue/fifo_order.py`.
#[test]
fn test_gen_behavior_std_libs_queue_fifo_order() {
    let out = jit_capture(r###"# /// script
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
"###);
    assert_output(&out, r###"fifo_order OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/queue/lifo_order.py`.
#[test]
fn test_gen_behavior_std_libs_queue_lifo_order() {
    let out = jit_capture(r###"# /// script
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
"###);
    assert_output(&out, r###"lifo_order OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/queue/maxsize_zero_is_unlimited.py`.
#[test]
fn test_gen_behavior_std_libs_queue_maxsize_zero_is_unlimited() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "behavior"
# case = "maxsize_zero_is_unlimited"
# subject = "queue.Queue"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_queue.py"
# status = "filled"
# ///
"""queue.Queue: maxsize=0 means unlimited capacity: 100 puts succeed, qsize()==100 and full() stays False"""
import queue

q = queue.Queue(maxsize=0)
for i in range(100):
    q.put(i)
assert q.qsize() == 100, f"unlimited qsize = {q.qsize()!r}"
assert q.full() is False, "unlimited queue never full"

print("maxsize_zero_is_unlimited OK")
"###);
    assert_output(&out, r###"maxsize_zero_is_unlimited OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/queue/priority_order.py`.
#[test]
fn test_gen_behavior_std_libs_queue_priority_order() {
    let out = jit_capture(r###"# /// script
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
"###);
    assert_output(&out, r###"priority_order OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/queue/qsize_empty_full_reflect_state.py`.
#[test]
fn test_gen_behavior_std_libs_queue_qsize_empty_full_reflect_state() {
    let out = jit_capture(r###"# /// script
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
"###);
    assert_output(&out, r###"qsize_empty_full_reflect_state OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/queue/simplequeue_fifo_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_queue_simplequeue_fifo_roundtrip() {
    let out = jit_capture(r###"# /// script
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
"###);
    assert_output(&out, r###"simplequeue_fifo_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/queue/task_done_join_unblocks.py`.
#[test]
fn test_gen_behavior_std_libs_queue_task_done_join_unblocks() {
    let out = jit_capture(r###"# /// script
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
"###);
    assert_output(&out, r###"task_done_join_unblocks OK
"###);
}
