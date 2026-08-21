# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "bufferedreader_is_callable"
# subject = "io.BufferedReader"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.BufferedReader: bufferedreader_is_callable (surface)."""
import io

assert callable(io.BufferedReader)
print("bufferedreader_is_callable OK")
