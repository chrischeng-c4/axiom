# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "errors"
# case = "missing_command_raises_filenotfound"
# subject = "subprocess.run"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.run: missing_command_raises_filenotfound (errors)."""
import subprocess

_raised = False
try:
    subprocess.run(['definitely_not_a_real_command_xyzzy'], capture_output=True)
except FileNotFoundError:
    _raised = True
assert _raised, "missing_command_raises_filenotfound: expected FileNotFoundError"
print("missing_command_raises_filenotfound OK")
