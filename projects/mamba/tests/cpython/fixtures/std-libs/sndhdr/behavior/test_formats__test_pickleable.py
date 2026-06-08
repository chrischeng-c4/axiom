# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sndhdr"
# dimension = "behavior"
# case = "test_formats__test_pickleable"
# subject = "cpython.test_sndhdr.TestFormats.test_pickleable"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sndhdr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sndhdr.py::TestFormats::test_pickleable
"""Auto-ported test: TestFormats::test_pickleable (CPython 3.12 oracle)."""


import pickle
import unittest
from test.support import findfile
from test.support import warnings_helper


sndhdr = warnings_helper.import_deprecated('sndhdr')


# --- test body ---
filename = findfile('sndhdr.aifc', subdir='sndhdrdata')
what = sndhdr.what(filename)
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    dump = pickle.dumps(what, proto)

    assert pickle.loads(dump) == what
print("TestFormats::test_pickleable: ok")
