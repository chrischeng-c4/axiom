# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "seek_end_present"
# subject = "io"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io: seek_end_present (surface)."""
import io

assert hasattr(io, "SEEK_END")
print("seek_end_present OK")
