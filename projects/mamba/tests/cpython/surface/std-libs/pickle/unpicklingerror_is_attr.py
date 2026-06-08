# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "unpicklingerror_is_attr"
# subject = "pickle"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pickle: unpicklingerror_is_attr (surface)."""
import pickle

assert hasattr(pickle, "UnpicklingError")
print("unpicklingerror_is_attr OK")
