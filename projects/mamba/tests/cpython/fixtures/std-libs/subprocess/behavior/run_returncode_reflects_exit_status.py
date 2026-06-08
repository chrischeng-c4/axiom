# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "run_returncode_reflects_exit_status"
# subject = "subprocess.run"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.run: CompletedProcess.returncode equals the child's process exit status (SystemExit(3) -> returncode 3)"""
import subprocess
import sys

_r = subprocess.run([sys.executable, "-c", "raise SystemExit(3)"])
assert _r.returncode == 3, f"exit(3) returncode = {_r.returncode!r}"
print("run_returncode_reflects_exit_status OK")
