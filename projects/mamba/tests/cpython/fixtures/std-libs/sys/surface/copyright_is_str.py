# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "copyright_is_str"
# subject = "sys.copyright"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.copyright: copyright_is_str (surface)."""
import sys

assert type(sys.copyright).__name__ == "str"
print("copyright_is_str OK")
