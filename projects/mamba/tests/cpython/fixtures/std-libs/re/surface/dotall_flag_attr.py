# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "dotall_flag_attr"
# subject = "re"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re: dotall_flag_attr (surface)."""
import re

assert hasattr(re, "DOTALL")
print("dotall_flag_attr OK")
