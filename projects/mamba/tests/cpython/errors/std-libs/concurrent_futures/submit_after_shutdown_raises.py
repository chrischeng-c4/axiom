# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "errors"
# case = "submit_after_shutdown_raises"
# subject = "concurrent.futures.Executor.submit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Executor.submit: submit_after_shutdown_raises (errors)."""
from concurrent.futures import ThreadPoolExecutor
_ex = ThreadPoolExecutor(max_workers=1)
_ex.shutdown(wait=True)

_raised = False
try:
    _ex.submit(lambda: 1)
except RuntimeError:
    _raised = True
assert _raised, "submit_after_shutdown_raises: expected RuntimeError"
print("submit_after_shutdown_raises OK")
