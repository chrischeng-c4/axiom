# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "propagate_false_stops"
# subject = "logging.Logger"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Logger: setting child.propagate=False stops a child record from reaching a handler on the parent"""
import logging

import io

_stream = io.StringIO()
_h = logging.StreamHandler(_stream)
_h.setLevel(logging.DEBUG)
_parent = logging.getLogger("test.behavior.parent5")
_parent.setLevel(logging.DEBUG)
_parent.addHandler(_h)
_child = logging.getLogger("test.behavior.parent5.child")
_child.propagate = False
_child.debug("no propagate")
_out = _stream.getvalue()
assert "no propagate" not in _out, f"propagate=False stops: {_out!r}"
_parent.removeHandler(_h)
_child.propagate = True  # reset shared module state
print("propagate_false_stops OK")
