# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "api_walk_tb_is_present"
# subject = "traceback.walk_tb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""traceback.walk_tb: api_walk_tb_is_present (surface)."""
import traceback

assert hasattr(traceback, "walk_tb")
print("api_walk_tb_is_present OK")
