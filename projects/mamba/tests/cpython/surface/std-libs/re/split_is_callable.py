# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "split_is_callable"
# subject = "re.split"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.split: split_is_callable (surface)."""
import re

assert callable(re.split)
print("split_is_callable OK")
