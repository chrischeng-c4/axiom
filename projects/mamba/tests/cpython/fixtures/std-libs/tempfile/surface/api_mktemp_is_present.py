# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "api_mktemp_is_present"
# subject = "tempfile.mktemp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tempfile.mktemp: api_mktemp_is_present (surface)."""
import tempfile

assert hasattr(tempfile, "mktemp")
print("api_mktemp_is_present OK")
