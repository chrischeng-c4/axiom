# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "errors"
# case = "nul_in_argv_raises_valueerror"
# subject = "subprocess.Popen"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.Popen: nul_in_argv_raises_valueerror (errors)."""
import subprocess
import sys

_raised = False
try:
    subprocess.Popen([sys.executable, '-c', 'pass#\x00'])
except ValueError:
    _raised = True
assert _raised, "nul_in_argv_raises_valueerror: expected ValueError"
print("nul_in_argv_raises_valueerror OK")
