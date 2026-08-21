# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "errors"
# case = "signal_none_handler_typeerror"
# subject = "signal.signal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.signal: signal_none_handler_typeerror (errors)."""
import signal

_raised = False
try:
    signal.signal(signal.SIGUSR1, None)
except TypeError:
    _raised = True
assert _raised, "signal_none_handler_typeerror: expected TypeError"
print("signal_none_handler_typeerror OK")
