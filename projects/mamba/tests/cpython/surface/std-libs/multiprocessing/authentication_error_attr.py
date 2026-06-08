# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "authentication_error_attr"
# subject = "multiprocessing"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""multiprocessing: authentication_error_attr (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "AuthenticationError")
print("authentication_error_attr OK")
