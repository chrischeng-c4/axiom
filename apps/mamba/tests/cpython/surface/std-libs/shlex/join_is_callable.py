# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "surface"
# case = "join_is_callable"
# subject = "shlex.join"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""shlex.join: join_is_callable (surface)."""
import shlex

assert callable(shlex.join)
print("join_is_callable OK")
