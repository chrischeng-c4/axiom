# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "stringio_subclasses_iobase"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: io.StringIO and io.BytesIO are both subclasses of io.IOBase"""
import io

assert issubclass(io.StringIO, io.IOBase), "StringIO subclass IOBase"
assert issubclass(io.BytesIO, io.IOBase), "BytesIO subclass IOBase"

print("stringio_subclasses_iobase OK")
