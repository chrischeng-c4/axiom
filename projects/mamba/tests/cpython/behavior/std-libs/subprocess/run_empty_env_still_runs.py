# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "run_empty_env_still_runs"
# subject = "subprocess.run"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.run: an empty env={} mapping is accepted and the child still runs to a clean exit"""
import subprocess
import sys

_r = subprocess.run([sys.executable, "-c", "pass"], env={})
assert _r.returncode == 0, f"empty env rc = {_r.returncode!r}"
print("run_empty_env_still_runs OK")
