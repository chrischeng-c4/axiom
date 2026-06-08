# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "finditer_is_callable"
# subject = "re.finditer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.finditer: finditer_is_callable (surface)."""
import re

assert callable(re.finditer)
print("finditer_is_callable OK")
