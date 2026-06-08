# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "surface"
# case = "pformat_is_callable"
# subject = "pprint.pformat"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pprint.pformat: pformat_is_callable (surface)."""
import pprint

assert callable(pprint.pformat)
print("pformat_is_callable OK")
