# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "errors"
# case = "signal_sigstop_oserror"
# subject = "signal.signal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.signal: signal_sigstop_oserror (errors)."""
import signal

_raised = False
try:
    signal.signal(signal.SIGSTOP, signal.SIG_IGN)
except OSError:
    _raised = True
assert _raised, "signal_sigstop_oserror: expected OSError"
print("signal_sigstop_oserror OK")
