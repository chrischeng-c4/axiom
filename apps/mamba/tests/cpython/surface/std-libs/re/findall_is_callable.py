# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "findall_is_callable"
# subject = "re.findall"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.findall: findall_is_callable (surface)."""
import re

assert callable(re.findall)
print("findall_is_callable OK")
