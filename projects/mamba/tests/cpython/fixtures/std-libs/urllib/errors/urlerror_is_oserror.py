# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "errors"
# case = "urlerror_is_oserror"
# subject = "urllib.error.URLError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.error.URLError: URLError is a subclass of OSError (the documented exception hierarchy root)"""
from urllib.error import URLError

assert issubclass(URLError, OSError), "URLError < OSError"

print("urlerror_is_oserror OK")
