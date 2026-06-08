# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "formatter_controls_output"
# subject = "logging.Formatter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Formatter: a Formatter('%(levelname)s|%(message)s') makes an ERROR record render as 'ERROR|formatted_msg' in the handler stream"""
import logging

import io

_stream = io.StringIO()
_h = logging.StreamHandler(_stream)
_fmt = logging.Formatter("%(levelname)s|%(message)s")
_h.setFormatter(_fmt)
_h.setLevel(logging.DEBUG)
_log = logging.getLogger("test.behavior.r3")
_log.setLevel(logging.DEBUG)
_log.addHandler(_h)
_log.error("formatted_msg")
_out = _stream.getvalue()
assert "ERROR|formatted_msg" in _out, f"formatted output = {_out!r}"
_log.removeHandler(_h)
print("formatter_controls_output OK")
