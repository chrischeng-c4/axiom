# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "surface"
# case = "call_is_callable"
# subject = "subprocess.call"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""subprocess.call: call_is_callable (surface)."""
import subprocess

assert callable(subprocess.call)
print("call_is_callable OK")
