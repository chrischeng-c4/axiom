# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "errors"
# case = "httpstatus_unknown_value_raises_valueerror"
# subject = "http.HTTPStatus"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.HTTPStatus: httpstatus_unknown_value_raises_valueerror (errors)."""
from http import HTTPStatus

_raised = False
try:
    HTTPStatus(999)
except ValueError:
    _raised = True
assert _raised, "httpstatus_unknown_value_raises_valueerror: expected ValueError"
print("httpstatus_unknown_value_raises_valueerror OK")
