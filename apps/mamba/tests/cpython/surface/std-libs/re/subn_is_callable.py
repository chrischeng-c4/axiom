# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "subn_is_callable"
# subject = "re.subn"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.subn: subn_is_callable (surface)."""
import re

assert callable(re.subn)
print("subn_is_callable OK")
