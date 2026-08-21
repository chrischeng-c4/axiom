# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "highest_protocol_attr"
# subject = "pickle"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pickle: highest_protocol_attr (surface)."""
import pickle

assert hasattr(pickle, "HIGHEST_PROTOCOL")
print("highest_protocol_attr OK")
