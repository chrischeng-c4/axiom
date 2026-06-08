# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "api_gettempprefixb_is_present"
# subject = "tempfile.gettempprefixb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tempfile.gettempprefixb: api_gettempprefixb_is_present (surface)."""
import tempfile

assert hasattr(tempfile, "gettempprefixb")
print("api_gettempprefixb_is_present OK")
