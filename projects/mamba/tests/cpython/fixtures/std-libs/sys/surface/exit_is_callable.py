# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "exit_is_callable"
# subject = "sys.exit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.exit: exit_is_callable (surface)."""
import sys

assert callable(sys.exit)
print("exit_is_callable OK")
