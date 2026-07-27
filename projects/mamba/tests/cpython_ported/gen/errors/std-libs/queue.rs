use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/queue/get_block_false_on_empty_raises.py`.
#[test]
fn test_gen_errors_std_libs_queue_get_block_false_on_empty_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "errors"
# case = "get_block_false_on_empty_raises"
# subject = "queue.Queue.get"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_queue.py"
# status = "filled"
# ///
"""queue.Queue.get: get_block_false_on_empty_raises (errors)."""
import queue

_raised = False
try:
    queue.Queue().get(block=False)
except queue.Empty:
    _raised = True
assert _raised, "get_block_false_on_empty_raises: expected queue.Empty"
print("get_block_false_on_empty_raises OK")
"###);
    assert_output(&out, r###"get_block_false_on_empty_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/queue/get_negative_timeout_raises.py`.
#[test]
fn test_gen_errors_std_libs_queue_get_negative_timeout_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "errors"
# case = "get_negative_timeout_raises"
# subject = "queue.Queue.get"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_queue.py"
# status = "filled"
# ///
"""queue.Queue.get: get_negative_timeout_raises (errors)."""
import queue

_raised = False
try:
    queue.Queue().get(timeout=-1)
except ValueError:
    _raised = True
assert _raised, "get_negative_timeout_raises: expected ValueError"
print("get_negative_timeout_raises OK")
"###);
    assert_output(&out, r###"get_negative_timeout_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/queue/get_nowait_on_empty_raises.py`.
#[test]
fn test_gen_errors_std_libs_queue_get_nowait_on_empty_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "errors"
# case = "get_nowait_on_empty_raises"
# subject = "queue.Queue.get_nowait"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_queue.py"
# status = "filled"
# ///
"""queue.Queue.get_nowait: get_nowait_on_empty_raises (errors)."""
import queue

_raised = False
try:
    queue.Queue().get_nowait()
except queue.Empty:
    _raised = True
assert _raised, "get_nowait_on_empty_raises: expected queue.Empty"
print("get_nowait_on_empty_raises OK")
"###);
    assert_output(&out, r###"get_nowait_on_empty_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/queue/get_timeout_on_empty_raises.py`.
#[test]
fn test_gen_errors_std_libs_queue_get_timeout_on_empty_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "errors"
# case = "get_timeout_on_empty_raises"
# subject = "queue.Queue.get"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_queue.py"
# status = "filled"
# ///
"""queue.Queue.get: get_timeout_on_empty_raises (errors)."""
import queue

_raised = False
try:
    queue.Queue().get(timeout=0.001)
except queue.Empty:
    _raised = True
assert _raised, "get_timeout_on_empty_raises: expected queue.Empty"
print("get_timeout_on_empty_raises OK")
"###);
    assert_output(&out, r###"get_timeout_on_empty_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/queue/put_negative_timeout_raises.py`.
#[test]
fn test_gen_errors_std_libs_queue_put_negative_timeout_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "errors"
# case = "put_negative_timeout_raises"
# subject = "queue.Queue.put"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_queue.py"
# status = "filled"
# ///
"""queue.Queue.put: put_negative_timeout_raises (errors)."""
import queue

_raised = False
try:
    queue.Queue(maxsize=1).put(1, timeout=-1)
except ValueError:
    _raised = True
assert _raised, "put_negative_timeout_raises: expected ValueError"
print("put_negative_timeout_raises OK")
"###);
    assert_output(&out, r###"put_negative_timeout_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/queue/put_nowait_on_full_raises.py`.
#[test]
fn test_gen_errors_std_libs_queue_put_nowait_on_full_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "errors"
# case = "put_nowait_on_full_raises"
# subject = "queue.Queue.put_nowait"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_queue.py"
# status = "filled"
# ///
"""queue.Queue.put_nowait: put_nowait_on_full_raises (errors)."""
import queue

_raised = False
try:
    (lambda q: (q.put_nowait('a'), q.put_nowait('b')))(queue.Queue(maxsize=1))
except queue.Full:
    _raised = True
assert _raised, "put_nowait_on_full_raises: expected queue.Full"
print("put_nowait_on_full_raises OK")
"###);
    assert_output(&out, r###"put_nowait_on_full_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/queue/task_done_too_many_raises.py`.
#[test]
fn test_gen_errors_std_libs_queue_task_done_too_many_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "errors"
# case = "task_done_too_many_raises"
# subject = "queue.Queue.task_done"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_queue.py"
# status = "filled"
# ///
"""queue.Queue.task_done: task_done_too_many_raises (errors)."""
import queue

_raised = False
try:
    (lambda q: (q.put('x'), q.get(), q.task_done(), q.task_done()))(queue.Queue())
except ValueError:
    _raised = True
assert _raised, "task_done_too_many_raises: expected ValueError"
print("task_done_too_many_raises OK")
"###);
    assert_output(&out, r###"task_done_too_many_raises OK
"###);
}
