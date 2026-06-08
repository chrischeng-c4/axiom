# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "imghdr"
# dimension = "behavior"
# case = "test_imghdr__test_data"
# subject = "cpython.test_imghdr.TestImghdr.test_data"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_imghdr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_imghdr.py::TestImghdr::test_data
"""Auto-ported test: TestImghdr::test_data (CPython 3.12 oracle)."""


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
for filename, expected in TEST_FILES:
    filename = findfile(filename, subdir='imghdrdata')

    assert imghdr.what(filename) == expected
    with open(filename, 'rb') as stream:

        assert imghdr.what(stream) == expected
    with open(filename, 'rb') as stream:
        data = stream.read()

    assert imghdr.what(None, data) == expected

    assert imghdr.what(None, bytearray(data)) == expected
print("TestImghdr::test_data: ok")
