# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "bdbquit_subclasses_exception"
# subject = "bdb.BdbQuit"
# kind = "semantic"
# xfail = "mamba bdb stub: BdbQuit is not a real Exception subclass (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.BdbQuit: BdbQuit is an Exception subclass usable for cleanly aborting a debug session"""
import bdb

assert issubclass(bdb.BdbQuit, Exception), "BdbQuit is an Exception subclass"

_caught = False
try:
    raise bdb.BdbQuit
except bdb.BdbQuit:
    _caught = True
assert _caught, "BdbQuit is raisable and catchable"

print("bdbquit_subclasses_exception OK")
