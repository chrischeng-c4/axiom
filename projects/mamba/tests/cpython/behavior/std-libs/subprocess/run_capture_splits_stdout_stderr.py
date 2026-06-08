# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "run_capture_splits_stdout_stderr"
# subject = "subprocess.run"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.run: capture_output=True captures the child's stdout and stderr into separate CompletedProcess fields"""
import subprocess
import sys

_r = subprocess.run(
    [sys.executable, "-c", "import sys; print('out'); print('err', file=sys.stderr)"],
    capture_output=True, text=True,
)
assert _r.stdout.strip() == "out", f"stdout = {_r.stdout!r}"
assert _r.stderr.strip() == "err", f"stderr = {_r.stderr!r}"
assert _r.returncode == 0, f"returncode = {_r.returncode!r}"
print("run_capture_splits_stdout_stderr OK")
