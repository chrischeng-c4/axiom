# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "unpickler_is_attr"
# subject = "pickle"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pickle: unpickler_is_attr (surface)."""
import pickle

assert hasattr(pickle, "Unpickler")
print("unpickler_is_attr OK")
