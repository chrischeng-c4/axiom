# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "errors"
# case = "contenttooshort_is_urlerror"
# subject = "urllib.error.ContentTooShortError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.error.ContentTooShortError: ContentTooShortError is a subclass of URLError"""
from urllib.error import URLError, ContentTooShortError

assert issubclass(ContentTooShortError, URLError), "ContentTooShortError < URLError"

caught = False
try:
    raise ContentTooShortError("download incomplete", b"partial")
except URLError as e:
    caught = True
    assert isinstance(e, ContentTooShortError), type(e)
assert caught, "ContentTooShortError raised and caught as URLError"
print("contenttooshort_is_urlerror OK")
