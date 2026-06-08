# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "bufferedwriter_is_callable"
# subject = "io.BufferedWriter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.BufferedWriter: bufferedwriter_is_callable (surface)."""
import io

assert callable(io.BufferedWriter)
print("bufferedwriter_is_callable OK")
