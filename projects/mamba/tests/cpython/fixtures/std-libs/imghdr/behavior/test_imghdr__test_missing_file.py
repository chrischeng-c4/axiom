# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "imghdr"
# dimension = "behavior"
# case = "test_imghdr__test_missing_file"
# subject = "cpython.test_imghdr.TestImghdr.test_missing_file"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_imghdr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_imghdr.py::TestImghdr::test_missing_file
"""Auto-ported test: TestImghdr::test_missing_file (CPython 3.12 oracle)."""


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
try:
    imghdr.what('missing')
    raise AssertionError('expected FileNotFoundError')
except FileNotFoundError:
    pass
print("TestImghdr::test_missing_file: ok")
