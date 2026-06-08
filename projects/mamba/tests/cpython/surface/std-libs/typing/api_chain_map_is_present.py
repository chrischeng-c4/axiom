# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_chain_map_is_present"
# subject = "typing.ChainMap"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.ChainMap: api_chain_map_is_present (surface)."""
import typing

assert hasattr(typing, "ChainMap")
print("api_chain_map_is_present OK")
