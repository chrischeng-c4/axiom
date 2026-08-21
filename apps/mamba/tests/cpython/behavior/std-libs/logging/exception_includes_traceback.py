# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "exception_includes_traceback"
# subject = "logging.Logger"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Logger: Logger.exception() inside an except block emits both the supplied message and the active exception's type name"""
import logging

import io

_stream = io.StringIO()
_h = logging.StreamHandler(_stream)
_h.setLevel(logging.DEBUG)
_log = logging.getLogger("test.behavior.r6")
_log.setLevel(logging.DEBUG)
_log.addHandler(_h)
try:
    raise ValueError("test_exc")
except ValueError:
    _log.exception("caught it")
_out = _stream.getvalue()
assert "caught it" in _out, f"exception message missing: {_out!r}"
assert "ValueError" in _out, f"exception type missing: {_out!r}"
_log.removeHandler(_h)
print("exception_includes_traceback OK")
