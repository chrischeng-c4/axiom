# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "surface"
# case = "check_call_is_callable"
# subject = "subprocess.check_call"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""subprocess.check_call: check_call_is_callable (surface)."""
import subprocess

assert callable(subprocess.check_call)
print("check_call_is_callable OK")
