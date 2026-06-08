# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "surface"
# case = "saferepr_is_callable"
# subject = "pprint.saferepr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pprint.saferepr: saferepr_is_callable (surface)."""
import pprint

assert callable(pprint.saferepr)
print("saferepr_is_callable OK")
