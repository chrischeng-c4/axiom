# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "errors"
# case = "pthread_sigmask_one_arg_typeerror"
# subject = "signal.pthread_sigmask"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.pthread_sigmask: pthread_sigmask_one_arg_typeerror (errors)."""
import signal

_raised = False
try:
    signal.pthread_sigmask(1)
except TypeError:
    _raised = True
assert _raised, "pthread_sigmask_one_arg_typeerror: expected TypeError"
print("pthread_sigmask_one_arg_typeerror OK")
