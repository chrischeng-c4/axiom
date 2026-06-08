# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "bufferingformatter_joins_batch"
# subject = "logging.BufferingFormatter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.BufferingFormatter: BufferingFormatter.format joins a record batch in order ('one','two' -> 'onetwo') and returns '' for an empty batch"""
import logging

records = [
    logging.makeLogRecord({"msg": "one"}),
    logging.makeLogRecord({"msg": "two"}),
]
bf = logging.BufferingFormatter()
assert bf.format([]) == "", "empty batch -> empty string"
assert bf.format(records) == "onetwo", "batch joined in order"
print("bufferingformatter_joins_batch OK")
