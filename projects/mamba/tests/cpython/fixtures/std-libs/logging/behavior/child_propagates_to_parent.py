# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "child_propagates_to_parent"
# subject = "logging.Logger"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Logger: a record on a dotted child logger propagates up to a handler installed on its ancestor"""
import logging

import io

_stream = io.StringIO()
_h = logging.StreamHandler(_stream)
_h.setLevel(logging.DEBUG)
_parent = logging.getLogger("test.behavior.parent")
_parent.setLevel(logging.DEBUG)
_parent.addHandler(_h)
_child = logging.getLogger("test.behavior.parent.child")
_child.debug("child msg")
_out = _stream.getvalue()
assert "child msg" in _out, f"propagation = {_out!r}"
_parent.removeHandler(_h)
print("child_propagates_to_parent OK")
