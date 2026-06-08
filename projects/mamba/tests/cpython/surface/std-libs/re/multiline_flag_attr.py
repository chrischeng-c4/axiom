# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "multiline_flag_attr"
# subject = "re"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re: multiline_flag_attr (surface)."""
import re

assert hasattr(re, "MULTILINE")
print("multiline_flag_attr OK")
