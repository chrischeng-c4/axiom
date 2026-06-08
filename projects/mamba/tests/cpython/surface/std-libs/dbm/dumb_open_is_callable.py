# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "surface"
# case = "dumb_open_is_callable"
# subject = "dbm.dumb.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.dumb.open: dumb_open_is_callable (surface)."""
import dbm.dumb

assert callable(dbm.dumb.open)
print("dumb_open_is_callable OK")
