# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "imghdr"
# dimension = "behavior"
# case = "test_imghdr__test_invalid_headers"
# subject = "cpython.test_imghdr.TestImghdr.test_invalid_headers"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_imghdr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_imghdr.py::TestImghdr::test_invalid_headers
"""Auto-ported test: TestImghdr::test_invalid_headers (CPython 3.12 oracle)."""


import io
import os
import pathlib
import unittest
import warnings
from test.support import findfile, warnings_helper
from test.support.os_helper import TESTFN, unlink


imghdr = warnings_helper.import_deprecated('imghdr')

TEST_FILES = (('python.png', 'png'), ('python.gif', 'gif'), ('python.bmp', 'bmp'), ('python.ppm', 'ppm'), ('python.pgm', 'pgm'), ('python.pbm', 'pbm'), ('python.jpg', 'jpeg'), ('python-raw.jpg', 'jpeg'), ('python.ras', 'rast'), ('python.sgi', 'rgb'), ('python.tiff', 'tiff'), ('python.xbm', 'xbm'), ('python.webp', 'webp'), ('python.exr', 'exr'))

class UnseekableIO(io.FileIO):

    def tell(self):
        raise io.UnsupportedOperation

    def seek(self, *args, **kwargs):
        raise io.UnsupportedOperation


# --- test body ---
for header in (b'\x89PN\r\n', b'\x01\xd9', b'Y\xa6', b'cutecat', b'000000JFI', b'GIF80'):

    assert imghdr.what(None, header) is None
print("TestImghdr::test_invalid_headers: ok")
