# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "behavior"
# case = "load_from_binary_file"
# subject = "tomllib.load"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_misc.py"
# status = "filled"
# ///
"""tomllib.load: tomllib.load reads from a binary file object (BytesIO and an on-disk tempfile opened 'rb') and yields the same dict tomllib.loads would"""
import tomllib
import io
import os
import tempfile

_content = b'key = "value"\nnum = 42\n'
_expected = {"key": "value", "num": 42}

# In-memory binary file object.
_d_mem = tomllib.load(io.BytesIO(_content))
assert _d_mem == _expected, f"BytesIO load = {_d_mem!r}"

# On-disk file opened in binary mode, inside a TemporaryDirectory.
with tempfile.TemporaryDirectory() as _tmp:
    _path = os.path.join(_tmp, "config.toml")
    with open(_path, "wb") as _wf:
        _wf.write(_content)
    with open(_path, "rb") as _rf:
        _d_disk = tomllib.load(_rf)
assert _d_disk == _expected, f"file load = {_d_disk!r}"

# Both paths agree with the string parser.
assert _d_mem == tomllib.loads(_content.decode()), "load vs loads divergence"

print("load_from_binary_file OK")
