# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "streamhandler_captures_record"
# subject = "logging.StreamHandler"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.StreamHandler: a StreamHandler over an io.StringIO captures an emitted INFO record's message into the underlying stream"""
import logging

import io

_stream = io.StringIO()
_handler = logging.StreamHandler(_stream)
_handler.setLevel(logging.DEBUG)
_logger = logging.getLogger("test.behavior.r1")
_logger.setLevel(logging.DEBUG)
_logger.addHandler(_handler)
_logger.info("hello from logger")
_out = _stream.getvalue()
assert "hello from logger" in _out, f"log captured = {_out!r}"
_logger.removeHandler(_handler)
print("streamhandler_captures_record OK")
