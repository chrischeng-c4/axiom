# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "surface"
# case = "api_z_trees_is_present"
# subject = "zlib.Z_TREES"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zlib.Z_TREES: api_z_trees_is_present (surface)."""
import zlib

assert hasattr(zlib, "Z_TREES")
print("api_z_trees_is_present OK")
