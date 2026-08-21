# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "imghdr"
# dimension = "behavior"
# case = "test_imghdr__test_register_test"
# subject = "cpython.test_imghdr.TestImghdr.test_register_test"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_imghdr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_imghdr.py::TestImghdr::test_register_test
"""Auto-ported test: TestImghdr::test_register_test (CPython 3.12 oracle)."""


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
def test_jumbo(h, file):
    if h.startswith(b'eggs'):
        return 'ham'
imghdr.tests.append(test_jumbo)
pass

assert imghdr.what(None, b'eggs') == 'ham'
print("TestImghdr::test_register_test: ok")
