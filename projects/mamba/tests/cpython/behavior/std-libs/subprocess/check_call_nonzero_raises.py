# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "check_call_nonzero_raises"
# subject = "subprocess.check_call"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.check_call: subprocess.check_call raises CalledProcessError whose .returncode echoes the child's non-zero exit"""
import subprocess
import sys

_raised = False
try:
    subprocess.check_call([sys.executable, "-c", "import sys; sys.exit(3)"])
except subprocess.CalledProcessError as exc:
    _raised = True
    assert exc.returncode == 3, f"exc.returncode = {exc.returncode!r}"
assert _raised, "check_call raises on non-zero"
print("check_call_nonzero_raises OK")
