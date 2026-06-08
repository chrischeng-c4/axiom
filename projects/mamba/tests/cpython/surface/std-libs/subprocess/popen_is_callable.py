# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "surface"
# case = "popen_is_callable"
# subject = "subprocess.Popen"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""subprocess.Popen: popen_is_callable (surface)."""
import subprocess

assert callable(subprocess.Popen)
print("popen_is_callable OK")
