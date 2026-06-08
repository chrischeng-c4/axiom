# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "surface"
# case = "error_is_tuple"
# subject = "dbm.error"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.error: error_is_tuple (surface)."""
import dbm

assert type(dbm.error).__name__ == "tuple"
print("error_is_tuple OK")
