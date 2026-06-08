# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "api_gettempprefix_is_present"
# subject = "tempfile.gettempprefix"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tempfile.gettempprefix: api_gettempprefix_is_present (surface)."""
import tempfile

assert hasattr(tempfile, "gettempprefix")
print("api_gettempprefix_is_present OK")
