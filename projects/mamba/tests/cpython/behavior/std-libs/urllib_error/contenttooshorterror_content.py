# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "behavior"
# case = "contenttooshorterror_content"
# subject = "urllib.error.ContentTooShortError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.error.ContentTooShortError: ContentTooShortError(message, content) keeps the partial content on .content and the message on .reason"""
from urllib.error import ContentTooShortError

e = ContentTooShortError("retrieval incomplete", b"partial data")
assert e.content == b"partial data", repr(e.content)
assert e.reason == "retrieval incomplete", repr(e.reason)
print("contenttooshorterror_content OK")
