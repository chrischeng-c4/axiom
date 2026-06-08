# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "thread_local_isolation"
# subject = "threading.local"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.local: a value stored on threading.local() in the main thread is not visible in a worker thread (per-thread storage isolation)"""
import threading

_local = threading.local()
_local.val = "main"
_result = []

def _check_local():
    # A new thread starts without _local.val.
    try:
        _ = _local.val
        _result.append("found")
    except AttributeError:
        _result.append("not found")

tl = threading.Thread(target=_check_local)
tl.start()
tl.join()
assert _result == ["not found"], f"thread local isolation = {_result!r}"

print("thread_local_isolation OK")
