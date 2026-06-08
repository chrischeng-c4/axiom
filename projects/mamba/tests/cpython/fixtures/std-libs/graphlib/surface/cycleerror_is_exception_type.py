# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "surface"
# case = "cycleerror_is_exception_type"
# subject = "graphlib.CycleError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""graphlib.CycleError: cycleerror_is_exception_type (surface)."""
import graphlib

assert hasattr(graphlib.CycleError, "__cause__")
print("cycleerror_is_exception_type OK")
