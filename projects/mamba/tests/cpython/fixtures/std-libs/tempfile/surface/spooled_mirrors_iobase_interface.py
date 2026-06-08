# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "spooled_mirrors_iobase_interface"
# subject = "tempfile.SpooledTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tempfile.SpooledTemporaryFile: SpooledTemporaryFile exposes the full IOBase method/attr subset (seek, read, write, close, closed, flush, readable, writable, __enter__, __exit__, __iter__, ...)"""
import tempfile

_iobase_attrs = {
    "fileno", "seek", "truncate", "close", "closed", "flush", "isatty",
    "readable", "readline", "readlines", "seekable", "tell", "writable",
    "writelines", "read", "write", "__enter__", "__exit__", "__iter__",
}
_spooled_attrs = set(dir(tempfile.SpooledTemporaryFile))
assert _iobase_attrs <= _spooled_attrs, \
    f"missing IOBase attrs = {_iobase_attrs - _spooled_attrs!r}"
print("spooled_mirrors_iobase_interface OK")
