# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "errors"
# case = "result_timeout_on_pending_raises"
# subject = "concurrent.futures.Future.result"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Future.result: Future.result(timeout) on a future whose task is still running raises concurrent.futures.TimeoutError before the task finishes"""
import concurrent.futures
import threading

# A task that blocks until released so the future stays pending across the
# short result() timeout window.
release = threading.Event()


def blocker():
    release.wait(10)
    return 1


with concurrent.futures.ThreadPoolExecutor(max_workers=1) as ex:
    fut = ex.submit(blocker)
    raised = False
    try:
        fut.result(timeout=0.01)
    except concurrent.futures.TimeoutError:
        raised = True
    assert raised, "result(timeout) on a pending future must raise TimeoutError"
    # Release the worker so the executor drains cleanly on context exit.
    release.set()

assert fut.result(timeout=5) == 1, "task still completes once released"

print("result_timeout_on_pending_raises OK")
