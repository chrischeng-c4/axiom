# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "api_mkdtemp_is_present"
# subject = "tempfile.mkdtemp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tempfile.mkdtemp: api_mkdtemp_is_present (surface)."""
import tempfile

assert hasattr(tempfile, "mkdtemp")
print("api_mkdtemp_is_present OK")
