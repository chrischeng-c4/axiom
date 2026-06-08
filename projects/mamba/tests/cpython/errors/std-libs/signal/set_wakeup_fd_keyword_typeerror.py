# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "errors"
# case = "set_wakeup_fd_keyword_typeerror"
# subject = "signal.set_wakeup_fd"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.set_wakeup_fd: set_wakeup_fd_keyword_typeerror (errors)."""
import signal

_raised = False
try:
    signal.set_wakeup_fd(signum=signal.SIGINT)
except TypeError:
    _raised = True
assert _raised, "set_wakeup_fd_keyword_typeerror: expected TypeError"
print("set_wakeup_fd_keyword_typeerror OK")
