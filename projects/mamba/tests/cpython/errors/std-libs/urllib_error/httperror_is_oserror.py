# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "errors"
# case = "httperror_is_oserror"
# subject = "urllib.error.HTTPError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.error.HTTPError: HTTPError is a subclass of OSError and is catchable as OSError when raised"""
from urllib.error import HTTPError

assert issubclass(HTTPError, OSError), "HTTPError < OSError"

caught = False
try:
    raise HTTPError("http://x.com/", 404, "Not Found", {}, None)
except OSError as e:
    caught = True
    assert isinstance(e, HTTPError), type(e)
assert caught, "HTTPError raised and caught as OSError"
print("httperror_is_oserror OK")
