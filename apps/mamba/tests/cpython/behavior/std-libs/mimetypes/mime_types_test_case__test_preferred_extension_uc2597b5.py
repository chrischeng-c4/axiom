# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "mime_types_test_case__test_preferred_extension_uc2597b5"
# subject = "cpython.test_mimetypes.MimeTypesTestCase.test_preferred_extension"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
import io
import mimetypes
import os
import sys
from platform import win32_edition
self_db = mimetypes.MimeTypes()

def check_extensions():
    assert mimetypes.guess_extension('application/octet-stream') == '.bin'
    assert mimetypes.guess_extension('application/postscript') == '.ps'
    assert mimetypes.guess_extension('application/vnd.apple.mpegurl') == '.m3u'
    assert mimetypes.guess_extension('application/vnd.ms-excel') == '.xls'
    assert mimetypes.guess_extension('application/vnd.ms-powerpoint') == '.ppt'
    assert mimetypes.guess_extension('application/x-texinfo') == '.texi'
    assert mimetypes.guess_extension('application/x-troff') == '.roff'
    assert mimetypes.guess_extension('application/xml') == '.xsl'
    assert mimetypes.guess_extension('audio/mpeg') == '.mp3'
    assert mimetypes.guess_extension('image/avif') == '.avif'
    assert mimetypes.guess_extension('image/jpeg') == '.jpg'
    assert mimetypes.guess_extension('image/tiff') == '.tiff'
    assert mimetypes.guess_extension('message/rfc822') == '.eml'
    assert mimetypes.guess_extension('text/html') == '.html'
    assert mimetypes.guess_extension('text/plain') == '.txt'
    assert mimetypes.guess_extension('text/x-rst') == '.rst'
    assert mimetypes.guess_extension('video/mpeg') == '.mpeg'
    assert mimetypes.guess_extension('video/quicktime') == '.mov'
check_extensions()
mimetypes.init()
check_extensions()

print("MimeTypesTestCase::test_preferred_extension: ok")
