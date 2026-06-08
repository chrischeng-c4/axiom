# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "check_output_input_feeds_stdin"
# subject = "subprocess.check_output"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.check_output: check_output(input=...) feeds the child's stdin and returns the resulting stdout bytes"""
import subprocess
import sys

_out = subprocess.check_output(
    [sys.executable, "-c", "import sys; sys.stdout.write(sys.stdin.read().upper())"],
    input=b"pear",
)
assert _out == b"PEAR", f"input upper = {_out!r}"
print("check_output_input_feeds_stdin OK")
