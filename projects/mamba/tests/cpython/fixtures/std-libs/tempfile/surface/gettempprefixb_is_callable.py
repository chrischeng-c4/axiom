# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "gettempprefixb_is_callable"
# subject = "tempfile.gettempprefixb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tempfile.gettempprefixb: gettempprefixb_is_callable (surface)."""
import tempfile

assert callable(tempfile.gettempprefixb)
print("gettempprefixb_is_callable OK")
