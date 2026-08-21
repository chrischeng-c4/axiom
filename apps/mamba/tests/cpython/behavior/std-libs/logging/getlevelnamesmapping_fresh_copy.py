# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "getlevelnamesmapping_fresh_copy"
# subject = "logging.getLevelNamesMapping"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.getLevelNamesMapping: getLevelNamesMapping returns a name->number dict containing the builtins, and hands back a fresh independent copy on every call"""
import logging

mapping = logging.getLevelNamesMapping()
assert mapping["DEBUG"] == logging.DEBUG, "mapping has DEBUG"
assert mapping["CRITICAL"] == logging.CRITICAL, "mapping has CRITICAL"
again = logging.getLevelNamesMapping()
assert mapping is not again, "fresh dict per call"
assert mapping == again, "equal contents"
mapping["BOGUS"] = 999
assert "BOGUS" not in logging.getLevelNamesMapping(), "copy is independent"
print("getlevelnamesmapping_fresh_copy OK")
