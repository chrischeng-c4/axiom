# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "call_returns_exit_code"
# subject = "subprocess.call"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.call: subprocess.call returns the child's exit code directly and never raises on a non-zero exit"""
import subprocess
import sys

rc = subprocess.call([sys.executable, "-c", "import sys; sys.exit(47)"])
assert rc == 47, f"call rc = {rc!r}"
print("call_returns_exit_code OK")
