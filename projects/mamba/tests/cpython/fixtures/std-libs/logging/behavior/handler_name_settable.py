# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "handler_name_settable"
# subject = "logging.Handler"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Handler: Handler.name is a settable attribute that round-trips through assignment and re-assignment"""
import logging

h = logging.Handler()
h.name = "generic"
assert h.name == "generic", "name set once"
h.name = "renamed"
assert h.name == "renamed", "name re-set"
print("handler_name_settable OK")
