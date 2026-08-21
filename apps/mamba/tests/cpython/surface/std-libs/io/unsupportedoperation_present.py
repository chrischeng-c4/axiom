# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "unsupportedoperation_present"
# subject = "io"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io: unsupportedoperation_present (surface)."""
import io

assert hasattr(io, "UnsupportedOperation")
print("unsupportedoperation_present OK")
