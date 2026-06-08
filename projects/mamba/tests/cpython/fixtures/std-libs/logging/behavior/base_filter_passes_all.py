# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "base_filter_passes_all"
# subject = "logging.Filter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Filter: a bare logging.Filter() passes every record (filter() returns truthy with no name restriction configured)"""
import logging

flt = logging.Filter()
assert flt.filter(logging.makeLogRecord({"name": "spam.eggs"})), "empty filter passes"
print("base_filter_passes_all OK")
