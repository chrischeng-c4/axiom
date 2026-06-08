# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "gettempdirb_is_callable"
# subject = "tempfile.gettempdirb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tempfile.gettempdirb: gettempdirb_is_callable (surface)."""
import tempfile

assert callable(tempfile.gettempdirb)
print("gettempdirb_is_callable OK")
