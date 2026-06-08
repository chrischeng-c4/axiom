# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "errors"
# case = "signal_out_of_range_signum_raises"
# subject = "signal.signal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.signal: signal_out_of_range_signum_raises (errors)."""
import signal

_raised = False
try:
    signal.signal(99999, signal.SIG_IGN)
except (ValueError, OSError):
    _raised = True
assert _raised, "signal_out_of_range_signum_raises: expected (ValueError, OSError)"
print("signal_out_of_range_signum_raises OK")
