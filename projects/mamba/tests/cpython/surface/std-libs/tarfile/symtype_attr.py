# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "symtype_attr"
# subject = "tarfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tarfile: symtype_attr (surface)."""
import tarfile

assert hasattr(tarfile, "SYMTYPE")
print("symtype_attr OK")
