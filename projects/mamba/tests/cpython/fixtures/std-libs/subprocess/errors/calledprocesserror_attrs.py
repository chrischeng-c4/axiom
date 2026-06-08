# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "errors"
# case = "calledprocesserror_attrs"
# subject = "subprocess.CalledProcessError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.CalledProcessError: a CalledProcessError raised by check=True exposes .returncode and a list .cmd"""
import subprocess
import sys

_raised = False
try:
    subprocess.run([sys.executable, "-c", "import sys; sys.exit(7)"],
                   check=True, capture_output=True)
except subprocess.CalledProcessError as exc:
    _raised = True
    assert exc.returncode == 7, f"exc.returncode = {exc.returncode!r}"
    assert isinstance(exc.cmd, list), f"exc.cmd type = {type(exc.cmd)!r}"
    assert exc.cmd[0] == sys.executable, f"exc.cmd[0] = {exc.cmd[0]!r}"
assert _raised, "check=True raises CalledProcessError on non-zero"
print("calledprocesserror_attrs OK")
