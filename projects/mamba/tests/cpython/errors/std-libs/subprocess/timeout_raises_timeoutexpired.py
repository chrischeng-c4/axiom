# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "errors"
# case = "timeout_raises_timeoutexpired"
# subject = "subprocess.TimeoutExpired"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.TimeoutExpired: a child that outlives the timeout= deadline makes subprocess.run raise TimeoutExpired"""
import subprocess
import sys

_raised = False
try:
    subprocess.run([sys.executable, "-c", "import time; time.sleep(10)"],
                   timeout=0.1, capture_output=True)
except subprocess.TimeoutExpired:
    _raised = True
assert _raised, "timeout raises TimeoutExpired"
print("timeout_raises_timeoutexpired OK")
