# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "errors"
# case = "non_integer_bufsize_raises_typeerror"
# subject = "subprocess.Popen"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.Popen: non_integer_bufsize_raises_typeerror (errors)."""
import subprocess
import sys

_raised = False
try:
    subprocess.Popen([sys.executable, '-c', 'pass'], 'orange')
except TypeError:
    _raised = True
assert _raised, "non_integer_bufsize_raises_typeerror: expected TypeError"
print("non_integer_bufsize_raises_typeerror OK")
