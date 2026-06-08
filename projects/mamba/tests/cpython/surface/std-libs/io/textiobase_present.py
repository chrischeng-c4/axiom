# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "textiobase_present"
# subject = "io"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io: textiobase_present (surface)."""
import io

assert hasattr(io, "TextIOBase")
print("textiobase_present OK")
