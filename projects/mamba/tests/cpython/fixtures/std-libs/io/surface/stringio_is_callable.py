# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "stringio_is_callable"
# subject = "io.StringIO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: stringio_is_callable (surface)."""
import io

assert callable(io.StringIO)
print("stringio_is_callable OK")
