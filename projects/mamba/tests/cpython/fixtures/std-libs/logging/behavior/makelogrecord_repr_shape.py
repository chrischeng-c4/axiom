# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "makelogrecord_repr_shape"
# subject = "logging.makeLogRecord"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.makeLogRecord: str(makeLogRecord({})) has the documented '<LogRecord: ...>' repr shape"""
import logging

rec = logging.makeLogRecord({})
text = str(rec)
assert text.startswith("<LogRecord: "), f"repr prefix: {text!r}"
assert text.endswith(">"), f"repr suffix: {text!r}"
print("makelogrecord_repr_shape OK")
