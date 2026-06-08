# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "errors"
# case = "exit_raises_systemexit"
# subject = "sys.exit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.exit: exit_raises_systemexit (errors)."""
import sys

_raised = False
try:
    sys.exit(42)
except SystemExit:
    _raised = True
assert _raised, "exit_raises_systemexit: expected SystemExit"
print("exit_raises_systemexit OK")
