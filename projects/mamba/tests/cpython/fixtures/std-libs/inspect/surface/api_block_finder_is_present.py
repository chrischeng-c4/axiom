# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_block_finder_is_present"
# subject = "inspect.BlockFinder"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.BlockFinder: api_block_finder_is_present (surface)."""
import inspect

assert hasattr(inspect, "BlockFinder")
print("api_block_finder_is_present OK")
