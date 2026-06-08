# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pdb"
# dimension = "surface"
# case = "api_runctx_is_present"
# subject = "pdb.runctx"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pdb.runctx: api_runctx_is_present (surface)."""
import pdb

assert hasattr(pdb, "runctx")
print("api_runctx_is_present OK")
