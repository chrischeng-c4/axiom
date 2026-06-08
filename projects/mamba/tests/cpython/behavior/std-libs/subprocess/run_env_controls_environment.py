# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "run_env_controls_environment"
# subject = "subprocess.run"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.run: the env= mapping controls the child process environment; a custom variable is visible inside the child"""
import os
import subprocess
import sys

_r = subprocess.run(
    [sys.executable, "-c", "import os; print(os.environ.get('MYVAR', 'missing'))"],
    capture_output=True, text=True,
    env={**os.environ, "MYVAR": "hello123"},
)
assert _r.stdout.strip() == "hello123", f"env var = {_r.stdout!r}"
print("run_env_controls_environment OK")
