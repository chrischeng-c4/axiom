# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "check_output_input_none_empty_stdin"
# subject = "subprocess.check_output"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.check_output: check_output(input=None, text=True) feeds no stdin, so the child reads an empty string"""
import subprocess
import sys

_out = subprocess.check_output(
    [sys.executable, "-c", "import sys; print('XX' if sys.stdin.read() else 'empty')"],
    input=None, text=True,
)
assert _out.strip() == "empty", f"input none = {_out!r}"
print("check_output_input_none_empty_stdin OK")
