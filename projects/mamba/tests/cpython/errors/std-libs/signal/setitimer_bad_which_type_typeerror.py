# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "errors"
# case = "setitimer_bad_which_type_typeerror"
# subject = "signal.setitimer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.setitimer: setitimer_bad_which_type_typeerror (errors)."""
import signal

_raised = False
try:
    signal.setitimer('not_int', 1.0)
except TypeError:
    _raised = True
assert _raised, "setitimer_bad_which_type_typeerror: expected TypeError"
print("setitimer_bad_which_type_typeerror OK")
