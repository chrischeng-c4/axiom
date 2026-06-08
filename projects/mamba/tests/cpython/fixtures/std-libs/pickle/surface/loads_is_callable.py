# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "loads_is_callable"
# subject = "pickle.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pickle.loads: loads_is_callable (surface)."""
import pickle

assert callable(pickle.loads)
print("loads_is_callable OK")
