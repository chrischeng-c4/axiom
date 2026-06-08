# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "surface"
# case = "pp_is_callable"
# subject = "pprint.pp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pprint.pp: pp_is_callable (surface)."""
import pprint

assert callable(pprint.pp)
print("pp_is_callable OK")
