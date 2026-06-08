# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "check_output_stderr_to_stdout"
# subject = "subprocess.check_output"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.check_output: check_output(stderr=STDOUT) folds the child's stderr into the captured stdout return value"""
import subprocess
import sys

_out = subprocess.check_output(
    [sys.executable, "-c", "import sys; sys.stderr.write('BDFL')"],
    stderr=subprocess.STDOUT,
)
assert _out == b"BDFL", f"stderr->stdout = {_out!r}"
print("check_output_stderr_to_stdout OK")
