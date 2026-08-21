# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "errors"
# case = "httperror_is_urlerror"
# subject = "urllib.error.HTTPError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.error.HTTPError: HTTPError is a subclass of URLError and is catchable as URLError when raised"""
from urllib.error import URLError, HTTPError

assert issubclass(HTTPError, URLError), "HTTPError < URLError"

caught = False
try:
    raise HTTPError("http://x.com/", 403, "Forbidden", {}, None)
except URLError as e:
    caught = True
    assert isinstance(e, HTTPError), type(e)
assert caught, "HTTPError raised and caught as URLError"
print("httperror_is_urlerror OK")
