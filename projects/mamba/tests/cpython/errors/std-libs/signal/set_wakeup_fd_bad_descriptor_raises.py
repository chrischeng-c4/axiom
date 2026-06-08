# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "errors"
# case = "set_wakeup_fd_bad_descriptor_raises"
# subject = "signal.set_wakeup_fd"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.set_wakeup_fd: set_wakeup_fd_bad_descriptor_raises (errors)."""
import signal

_raised = False
try:
    signal.set_wakeup_fd(2 ** 30)
except (ValueError, OSError):
    _raised = True
assert _raised, "set_wakeup_fd_bad_descriptor_raises: expected (ValueError, OSError)"
print("set_wakeup_fd_bad_descriptor_raises OK")
