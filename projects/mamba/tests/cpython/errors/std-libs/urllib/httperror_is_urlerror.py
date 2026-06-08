# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "errors"
# case = "httperror_is_urlerror"
# subject = "urllib.error.HTTPError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.error.HTTPError: HTTPError is a subclass of URLError (and therefore of OSError)"""
from urllib.error import URLError, HTTPError

assert issubclass(HTTPError, URLError), "HTTPError < URLError"
assert issubclass(HTTPError, OSError), "HTTPError < OSError"

print("httperror_is_urlerror OK")
