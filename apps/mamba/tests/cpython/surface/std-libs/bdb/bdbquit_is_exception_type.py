# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "surface"
# case = "bdbquit_is_exception_type"
# subject = "bdb.BdbQuit"
# kind = "mechanical"
# xfail = "mamba bdb stub: BdbQuit is not a real Exception subclass (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.BdbQuit: bdbquit_is_exception_type (surface)."""
import bdb

assert type(bdb.BdbQuit).__name__ == "type"
print("bdbquit_is_exception_type OK")
