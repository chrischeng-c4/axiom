# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "load_is_callable"
# subject = "pickle.load"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pickle.load: load_is_callable (surface)."""
import pickle

assert callable(pickle.load)
print("load_is_callable OK")
