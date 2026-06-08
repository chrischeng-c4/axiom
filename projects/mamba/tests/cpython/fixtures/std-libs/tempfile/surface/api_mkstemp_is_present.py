# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "api_mkstemp_is_present"
# subject = "tempfile.mkstemp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tempfile.mkstemp: api_mkstemp_is_present (surface)."""
import tempfile

assert hasattr(tempfile, "mkstemp")
print("api_mkstemp_is_present OK")
