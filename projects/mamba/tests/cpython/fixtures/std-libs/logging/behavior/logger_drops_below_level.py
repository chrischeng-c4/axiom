# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "logger_drops_below_level"
# subject = "logging.Logger"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Logger: a logger set to WARNING drops DEBUG/INFO records but emits WARNING records to its handler's stream"""
import logging

import io

_stream = io.StringIO()
_h = logging.StreamHandler(_stream)
_h.setLevel(logging.DEBUG)
_log = logging.getLogger("test.behavior.r2")
_log.setLevel(logging.WARNING)
_log.addHandler(_h)
_log.debug("should be dropped")
_log.info("also dropped")
_log.warning("should appear")
_out = _stream.getvalue()
assert "should be dropped" not in _out, "DEBUG dropped at WARNING level"
assert "also dropped" not in _out, "INFO dropped at WARNING level"
assert "should appear" in _out, "WARNING appears"
_log.removeHandler(_h)
print("logger_drops_below_level OK")
