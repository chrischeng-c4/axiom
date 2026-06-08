# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "streamhandler_setstream_returns_prior"
# subject = "logging.StreamHandler"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.StreamHandler: StreamHandler defaults to sys.stderr; setStream returns the previously attached stream, and returns None when the stream is unchanged"""
import logging

import io
import sys

sh = logging.StreamHandler()
buf = io.StringIO()
old = sh.setStream(buf)
assert old is sys.stderr, "default stream is stderr"
back = sh.setStream(old)
assert back is buf, "setStream returns prior stream"
noop = sh.setStream(old)
assert noop is None, "no-op setStream returns None"
print("streamhandler_setstream_returns_prior OK")
