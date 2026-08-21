# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "errors"
# case = "pthread_sigmask_bad_how_oserror"
# subject = "signal.pthread_sigmask"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.pthread_sigmask: pthread_sigmask_bad_how_oserror (errors)."""
import signal

_raised = False
try:
    signal.pthread_sigmask(1700, [])
except OSError:
    _raised = True
assert _raised, "pthread_sigmask_bad_how_oserror: expected OSError"
print("pthread_sigmask_bad_how_oserror OK")
