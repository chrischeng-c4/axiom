# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "surface"
# case = "split_is_callable"
# subject = "shlex.split"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""shlex.split: split_is_callable (surface)."""
import shlex

assert callable(shlex.split)
print("split_is_callable OK")
