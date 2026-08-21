# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "surface"
# case = "capwords_is_callable"
# subject = "string.capwords"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.capwords: capwords_is_callable (surface)."""
import string

assert callable(string.capwords)
print("capwords_is_callable OK")
