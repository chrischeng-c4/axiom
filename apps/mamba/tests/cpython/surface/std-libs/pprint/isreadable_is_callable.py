# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "surface"
# case = "isreadable_is_callable"
# subject = "pprint.isreadable"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pprint.isreadable: isreadable_is_callable (surface)."""
import pprint

assert callable(pprint.isreadable)
print("isreadable_is_callable OK")
