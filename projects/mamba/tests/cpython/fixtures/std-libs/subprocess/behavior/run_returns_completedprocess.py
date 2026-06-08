# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "run_returns_completedprocess"
# subject = "subprocess.run"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.run: subprocess.run(capture_output=True, text=True) returns a CompletedProcess with returncode 0, captured stdout, and accessible str stderr for a simple echo"""
import subprocess

_r = subprocess.run(["echo", "hello"], capture_output=True, text=True)
assert isinstance(_r, subprocess.CompletedProcess), f"run type = {type(_r)!r}"
assert _r.returncode == 0, f"echo returncode = {_r.returncode!r}"
assert "hello" in _r.stdout, f"stdout = {_r.stdout!r}"
assert isinstance(_r.stderr, str), f"stderr type = {type(_r.stderr)!r}"
assert _r.args == ["echo", "hello"], f"args = {_r.args!r}"
print("run_returns_completedprocess OK")
