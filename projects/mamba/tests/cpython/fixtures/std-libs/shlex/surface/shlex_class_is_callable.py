# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "surface"
# case = "shlex_class_is_callable"
# subject = "shlex.shlex"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""shlex.shlex: shlex_class_is_callable (surface)."""
import shlex

assert callable(shlex.shlex)
print("shlex_class_is_callable OK")
