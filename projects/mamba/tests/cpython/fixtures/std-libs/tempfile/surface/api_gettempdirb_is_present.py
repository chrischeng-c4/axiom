# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "api_gettempdirb_is_present"
# subject = "tempfile.gettempdirb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tempfile.gettempdirb: api_gettempdirb_is_present (surface)."""
import tempfile

assert hasattr(tempfile, "gettempdirb")
print("api_gettempdirb_is_present OK")
