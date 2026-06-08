# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "errors"
# case = "broken_executor_is_runtimeerror"
# subject = "concurrent.futures.BrokenExecutor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.BrokenExecutor: concurrent.futures.BrokenExecutor is a subclass of RuntimeError"""
import concurrent.futures

assert issubclass(concurrent.futures.BrokenExecutor, RuntimeError), "BrokenExecutor is a RuntimeError"
# A RuntimeError handler therefore catches a raised BrokenExecutor.
raised = False
try:
    raise concurrent.futures.BrokenExecutor("pool broke")
except RuntimeError as e:
    raised = isinstance(e, concurrent.futures.BrokenExecutor)
assert raised, "BrokenExecutor caught as RuntimeError"

print("broken_executor_is_runtimeerror OK")
