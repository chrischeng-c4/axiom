# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "bytesio_instance_is_bufferediobase"
# subject = "io.BytesIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.BytesIO: io.BytesIO() is an instance of io.BytesIO and of io.BufferedIOBase"""
import io

_bio = io.BytesIO()
assert isinstance(_bio, io.BytesIO), f"BytesIO type = {type(_bio)!r}"
assert isinstance(_bio, io.BufferedIOBase), "BytesIO is BufferedIOBase"

print("bytesio_instance_is_bufferediobase OK")
