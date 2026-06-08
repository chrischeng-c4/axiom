# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "surface"
# case = "run_is_callable"
# subject = "subprocess.run"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""subprocess.run: run_is_callable (surface)."""
import subprocess

assert callable(subprocess.run)
print("run_is_callable OK")
