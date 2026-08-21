# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "regtype_attr"
# subject = "tarfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tarfile: regtype_attr (surface)."""
import tarfile

assert hasattr(tarfile, "REGTYPE")
print("regtype_attr OK")
