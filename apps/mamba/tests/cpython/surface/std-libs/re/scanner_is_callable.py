# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "scanner_is_callable"
# subject = "re.Scanner"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.Scanner: scanner_is_callable (surface)."""
import re

assert callable(re.Scanner)
print("scanner_is_callable OK")
