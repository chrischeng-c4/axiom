# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "iobase_present"
# subject = "io"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io: iobase_present (surface)."""
import io

assert hasattr(io, "IOBase")
print("iobase_present OK")
