# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "stringio_instance_is_textiobase"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: io.StringIO() is an instance of io.StringIO and of io.TextIOBase"""
import io

_sio = io.StringIO()
assert isinstance(_sio, io.StringIO), f"StringIO type = {type(_sio)!r}"
assert isinstance(_sio, io.TextIOBase), "StringIO is TextIOBase"

print("stringio_instance_is_textiobase OK")
