# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_end_of_block_is_present"
# subject = "inspect.EndOfBlock"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.EndOfBlock: api_end_of_block_is_present (surface)."""
import inspect

assert hasattr(inspect, "EndOfBlock")
print("api_end_of_block_is_present OK")
