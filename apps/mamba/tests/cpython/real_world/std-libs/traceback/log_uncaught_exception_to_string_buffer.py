# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "real_world"
# case = "log_uncaught_exception_to_string_buffer"
# subject = "traceback.print_exc"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.print_exc: an app's error handler logs an uncaught exception: catch it, traceback.print_exc(file=buf) into a StringIO 'log', and confirm the captured exception type and message are present in the log text"""
import io
import traceback


def handle_request(payload):
    # A toy request handler that fails on bad input.
    return payload["missing_key"]


_log = io.StringIO()
try:
    handle_request({})
except KeyError:
    # Application-level error handler: capture the traceback into the log.
    traceback.print_exc(file=_log)

_log_text = _log.getvalue()
assert "Traceback (most recent call last):" in _log_text, f"log header: {_log_text!r}"
assert "KeyError" in _log_text, f"log has exception type: {_log_text!r}"
assert "missing_key" in _log_text, f"log has exception message: {_log_text!r}"

print("log_uncaught_exception_to_string_buffer OK")
