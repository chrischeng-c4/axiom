use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/multiprocessing/get_context_unknown_method_raises.py`.
#[test]
fn test_gen_errors_std_libs_multiprocessing_get_context_unknown_method_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "errors"
# case = "get_context_unknown_method_raises"
# subject = "multiprocessing.get_context"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/_test_multiprocessing.py"
# status = "filled"
# ///
"""multiprocessing.get_context: get_context_unknown_method_raises (errors)."""
import multiprocessing

_raised = False
try:
    multiprocessing.get_context('no_such_method')
except ValueError:
    _raised = True
assert _raised, "get_context_unknown_method_raises: expected ValueError"
print("get_context_unknown_method_raises OK")
"###);
    assert_output(&out, r###"get_context_unknown_method_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/multiprocessing/queue_get_nowait_empty_raises.py`.
#[test]
fn test_gen_errors_std_libs_multiprocessing_queue_get_nowait_empty_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "errors"
# case = "queue_get_nowait_empty_raises"
# subject = "multiprocessing.Queue"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/_test_multiprocessing.py"
# status = "filled"
# ///
"""multiprocessing.Queue: queue_get_nowait_empty_raises (errors)."""
import multiprocessing
import queue as _queue
_q = multiprocessing.Queue()

_raised = False
try:
    _q.get_nowait()
except _queue.Empty:
    _raised = True
assert _raised, "queue_get_nowait_empty_raises: expected _queue.Empty"
print("queue_get_nowait_empty_raises OK")
"###);
    assert_output(&out, r###"queue_get_nowait_empty_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/multiprocessing/queue_put_nowait_full_raises.py`.
#[test]
fn test_gen_errors_std_libs_multiprocessing_queue_put_nowait_full_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "errors"
# case = "queue_put_nowait_full_raises"
# subject = "multiprocessing.Queue"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/_test_multiprocessing.py"
# status = "filled"
# ///
"""multiprocessing.Queue: queue_put_nowait_full_raises (errors)."""
import multiprocessing
import queue as _queue
_q = multiprocessing.Queue(maxsize=1)
_q.put_nowait('a')

_raised = False
try:
    _q.put_nowait('b')
except _queue.Full:
    _raised = True
assert _raised, "queue_put_nowait_full_raises: expected _queue.Full"
print("queue_put_nowait_full_raises OK")
"###);
    assert_output(&out, r###"queue_put_nowait_full_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/multiprocessing/value_bad_typecode_raises.py`.
#[test]
fn test_gen_errors_std_libs_multiprocessing_value_bad_typecode_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "errors"
# case = "value_bad_typecode_raises"
# subject = "multiprocessing.Value"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""multiprocessing.Value: value_bad_typecode_raises (errors)."""
import multiprocessing

_raised = False
try:
    multiprocessing.Value('not_a_typecode')
except (AttributeError, TypeError):
    _raised = True
assert _raised, "value_bad_typecode_raises: expected (AttributeError, TypeError)"
print("value_bad_typecode_raises OK")
"###);
    assert_output(&out, r###"value_bad_typecode_raises OK
"###);
}
