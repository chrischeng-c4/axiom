# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "check_output_returns_stdout_bytes"
# subject = "subprocess.check_output"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.check_output: check_output returns the child's stdout as bytes for a simple printing child"""
import subprocess
import sys

_out = subprocess.check_output([sys.executable, "-c", "print('check_out')"])
assert isinstance(_out, bytes), f"check_output type = {type(_out)!r}"
assert b"check_out" in _out, f"check_output = {_out!r}"
print("check_output_returns_stdout_bytes OK")
