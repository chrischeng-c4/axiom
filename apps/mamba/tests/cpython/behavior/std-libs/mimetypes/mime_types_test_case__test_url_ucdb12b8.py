# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "mime_types_test_case__test_url_ucdb12b8"
# subject = "cpython.test_mimetypes.MimeTypesTestCase.test_url"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import io
import mimetypes
import os
import sys
from platform import win32_edition
self_db = mimetypes.MimeTypes()
result = self_db.guess_type('http://host.html')
msg = 'URL only has a host name, not a file'
assert result == (None, None)
result = self_db.guess_type('http://example.com/host.html')
msg = 'Should be text/html'
assert result == ('text/html', None)
result = self_db.guess_type('http://example.com/host.html#x.tar')
assert result == ('text/html', None)
result = self_db.guess_type('http://example.com/host.html?q=x.tar')
assert result == ('text/html', None)

print("MimeTypesTestCase::test_url: ok")
