# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "chain_is_callable"
# subject = "itertools.chain"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""itertools.chain: chain_is_callable (surface)."""
import itertools

assert callable(itertools.chain)
print("chain_is_callable OK")
