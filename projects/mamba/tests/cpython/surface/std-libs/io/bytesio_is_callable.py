# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "bytesio_is_callable"
# subject = "io.BytesIO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.BytesIO: bytesio_is_callable (surface)."""
import io

assert callable(io.BytesIO)
print("bytesio_is_callable OK")
