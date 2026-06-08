# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "errors"
# case = "pthread_sigmask_zero_valueerror"
# subject = "signal.pthread_sigmask"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.pthread_sigmask: pthread_sigmask_zero_valueerror (errors)."""
import signal

_raised = False
try:
    signal.pthread_sigmask(signal.SIG_BLOCK, [0])
except ValueError:
    _raised = True
assert _raised, "pthread_sigmask_zero_valueerror: expected ValueError"
print("pthread_sigmask_zero_valueerror OK")
