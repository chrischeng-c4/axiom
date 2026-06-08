# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "surface"
# case = "run_exitfuncs_is_callable"
# subject = "atexit._run_exitfuncs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""atexit._run_exitfuncs: run_exitfuncs_is_callable (surface)."""
import atexit

assert callable(atexit._run_exitfuncs)
print("run_exitfuncs_is_callable OK")
