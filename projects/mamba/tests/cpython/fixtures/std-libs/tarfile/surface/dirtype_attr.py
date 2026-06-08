# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "dirtype_attr"
# subject = "tarfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tarfile: dirtype_attr (surface)."""
import tarfile

assert hasattr(tarfile, "DIRTYPE")
print("dirtype_attr OK")
