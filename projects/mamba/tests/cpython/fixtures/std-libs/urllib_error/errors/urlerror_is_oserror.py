# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "errors"
# case = "urlerror_is_oserror"
# subject = "urllib.error.URLError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.error.URLError: URLError is a subclass of OSError (documented exception hierarchy root) and is raiseable/catchable as OSError"""
from urllib.error import URLError

assert issubclass(URLError, OSError), "URLError < OSError"

caught = False
try:
    raise URLError("connection refused")
except OSError as e:
    caught = True
    assert isinstance(e, URLError), type(e)
assert caught, "URLError raised and caught as OSError"
print("urlerror_is_oserror OK")
