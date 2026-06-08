# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "check_call_zero_returns_zero"
# subject = "subprocess.check_call"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.check_call: subprocess.check_call returns 0 and does not raise when the child exits cleanly"""
import subprocess
import sys

rc = subprocess.check_call([sys.executable, "-c", "pass"])
assert rc == 0, f"check_call rc = {rc!r}"
print("check_call_zero_returns_zero OK")
