# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "surface"
# case = "dumb_error_subclasses_oserror"
# subject = "dbm.dumb.error"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dbm.py"
# status = "filled"
# ///
"""dbm.dumb.error: dbm.dumb.error is the dumb-backend exception type and subclasses OSError"""
import dbm.dumb

assert issubclass(dbm.dumb.error, OSError), f"dumb.error < OSError: {dbm.dumb.error!r}"
print("dumb_error_subclasses_oserror OK")
