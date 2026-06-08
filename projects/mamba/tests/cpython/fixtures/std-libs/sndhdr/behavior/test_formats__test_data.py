# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sndhdr"
# dimension = "behavior"
# case = "test_formats__test_data"
# subject = "cpython.test_sndhdr.TestFormats.test_data"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sndhdr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sndhdr.py::TestFormats::test_data
"""Auto-ported test: TestFormats::test_data (CPython 3.12 oracle)."""


from test.support import findfile
from test.support import warnings_helper


sndhdr = warnings_helper.import_deprecated("sndhdr")


for name, expected in (
    ("sndhdr.8svx", ("8svx", 0, 1, 0, 8)),
    ("sndhdr.aifc", ("aifc", 44100, 2, 5, 16)),
    ("sndhdr.aiff", ("aiff", 44100, 2, 5, 16)),
    ("sndhdr.au", ("au", 44100, 2, 5.0, 16)),
    ("sndhdr.hcom", ("hcom", 22050.0, 1, -1, 8)),
    ("sndhdr.sndt", ("sndt", 44100, 1, 5, 8)),
    ("sndhdr.voc", ("voc", 0, 1, -1, 8)),
    ("sndhdr.wav", ("wav", 44100, 2, 5, 16)),
):
    filename = findfile(name, subdir="sndhdrdata")
    what = sndhdr.what(filename)
    assert what is not None, filename
    assert what == expected
    assert what.filetype == expected[0]
    assert what.framerate == expected[1]
    assert what.nchannels == expected[2]
    assert what.nframes == expected[3]
    assert what.sampwidth == expected[4]

print("TestFormats::test_data: ok")
