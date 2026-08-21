# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "write_read_text_roundtrip"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: in a TemporaryDirectory, Path.write_text/read_text round-trip the same string and the file then reports exists()/is_file()"""
import pathlib

import tempfile
Path = pathlib.Path

with tempfile.TemporaryDirectory() as _tmpdir:
    _f = Path(_tmpdir) / "test.txt"
    _f.write_text("hello world")
    _content = _f.read_text()
    assert _content == "hello world", f"read_text = {_content!r}"
    assert _f.exists(), "file exists"
    assert _f.is_file(), "is file"
print("write_read_text_roundtrip OK")
