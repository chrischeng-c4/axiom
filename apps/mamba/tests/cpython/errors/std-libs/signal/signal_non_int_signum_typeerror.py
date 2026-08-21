# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "errors"
# case = "signal_non_int_signum_typeerror"
# subject = "signal.signal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.signal: signal_non_int_signum_typeerror (errors)."""
import signal

_raised = False
try:
    signal.signal('not_a_signum', signal.SIG_IGN)
except TypeError:
    _raised = True
assert _raised, "signal_non_int_signum_typeerror: expected TypeError"
print("signal_non_int_signum_typeerror OK")
