# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "errors"
# case = "check_true_raises_calledprocesserror"
# subject = "subprocess.CalledProcessError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.CalledProcessError: subprocess.run(check=True) on a non-zero exit raises CalledProcessError whose .returncode echoes the child's exit code"""
import subprocess
import sys

_raised = False
try:
    subprocess.run([sys.executable, "-c", "raise SystemExit(1)"], check=True)
except subprocess.CalledProcessError as exc:
    _raised = True
    assert exc.returncode == 1, f"CalledProcessError.returncode = {exc.returncode!r}"
assert _raised, "check=True raises on non-zero"
print("check_true_raises_calledprocesserror OK")
