# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "compile_is_callable"
# subject = "re.compile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.compile: compile_is_callable (surface)."""
import re

assert callable(re.compile)
print("compile_is_callable OK")
