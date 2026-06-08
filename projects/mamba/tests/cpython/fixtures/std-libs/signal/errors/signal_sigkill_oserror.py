# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "errors"
# case = "signal_sigkill_oserror"
# subject = "signal.signal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.signal: signal_sigkill_oserror (errors)."""
import signal

_raised = False
try:
    signal.signal(signal.SIGKILL, signal.SIG_IGN)
except OSError:
    _raised = True
assert _raised, "signal_sigkill_oserror: expected OSError"
print("signal_sigkill_oserror OK")
