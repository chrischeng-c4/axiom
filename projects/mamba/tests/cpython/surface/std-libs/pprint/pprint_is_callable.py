# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "surface"
# case = "pprint_is_callable"
# subject = "pprint.pprint"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pprint.pprint: pprint_is_callable (surface)."""
import pprint

assert callable(pprint.pprint)
print("pprint_is_callable OK")
