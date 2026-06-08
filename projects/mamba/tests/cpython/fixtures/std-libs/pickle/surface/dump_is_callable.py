# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "dump_is_callable"
# subject = "pickle.dump"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pickle.dump: dump_is_callable (surface)."""
import pickle

assert callable(pickle.dump)
print("dump_is_callable OK")
