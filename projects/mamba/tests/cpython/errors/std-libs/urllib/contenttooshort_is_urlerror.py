# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
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

print("contenttooshort_is_urlerror OK")
