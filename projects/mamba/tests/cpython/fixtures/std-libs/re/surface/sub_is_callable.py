# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "sub_is_callable"
# subject = "re.sub"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.sub: sub_is_callable (surface)."""
import re

assert callable(re.sub)
print("sub_is_callable OK")
