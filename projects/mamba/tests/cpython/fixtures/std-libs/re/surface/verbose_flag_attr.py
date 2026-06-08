# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "verbose_flag_attr"
# subject = "re"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re: verbose_flag_attr (surface)."""
import re

assert hasattr(re, "VERBOSE")
print("verbose_flag_attr OK")
