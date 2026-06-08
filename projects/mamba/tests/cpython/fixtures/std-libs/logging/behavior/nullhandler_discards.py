# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "nullhandler_discards"
# subject = "logging.NullHandler"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.NullHandler: a logger whose only handler is NullHandler discards records silently without raising"""
import logging

_log = logging.getLogger("test.behavior.null")
_log.setLevel(logging.DEBUG)
_log.addHandler(logging.NullHandler())
_log.info("silent")  # should not raise, just discarded
print("nullhandler_discards OK")
