# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "run_check_returncode_zero_no_raise"
# subject = "subprocess.run"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.run: CompletedProcess.check_returncode() does not raise when the child exited 0"""
import subprocess
import sys

_r = subprocess.run([sys.executable, "-c", "pass"])
assert _r.returncode == 0, f"returncode = {_r.returncode!r}"
_r.check_returncode()  # zero exit -> no raise
print("run_check_returncode_zero_no_raise OK")
