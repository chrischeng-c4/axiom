# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "getlogger_caches_by_name"
# subject = "logging.getLogger"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.getLogger: getLogger(name) returns a logging.Logger, and a second call with the same name returns the identical cached instance"""
import logging

_log = logging.getLogger("test.surface")
assert isinstance(_log, logging.Logger), f"getLogger type = {type(_log)!r}"
_log2 = logging.getLogger("test.surface")
assert _log is _log2, "getLogger caches by name"
print("getlogger_caches_by_name OK")
