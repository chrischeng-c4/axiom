# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "platform_is_str"
# subject = "sys.platform"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.platform: platform_is_str (surface)."""
import sys

assert type(sys.platform).__name__ == "str"
print("platform_is_str OK")
