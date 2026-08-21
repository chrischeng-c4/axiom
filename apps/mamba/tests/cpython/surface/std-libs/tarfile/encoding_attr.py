# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "encoding_attr"
# subject = "tarfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tarfile: encoding_attr (surface)."""
import tarfile

assert hasattr(tarfile, "ENCODING")
print("encoding_attr OK")
