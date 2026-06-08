# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "api_chain_is_present"
# subject = "itertools.chain"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""itertools.chain: api_chain_is_present (surface)."""
import itertools

assert hasattr(itertools, "chain")
print("api_chain_is_present OK")
