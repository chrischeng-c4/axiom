# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "errors"
# case = "kill_bad_pid_raises"
# subject = "os.kill"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.kill: kill_bad_pid_raises (errors)."""
import os

_raised = False
try:
    os.kill(99999999, 0)
except OSError:
    _raised = True
assert _raised, "kill_bad_pid_raises: expected OSError"
print("kill_bad_pid_raises OK")
