# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "surface"
# case = "struct_size_constants"
# subject = "zipfile.sizeEndCentDir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.sizeEndCentDir: fixed on-disk structure sizes: sizeEndCentDir==22, sizeCentralDir==46, sizeEndCentDir64==56, sizeEndCentDir64Locator==20"""
import zipfile

assert zipfile.sizeEndCentDir == 22, f"sizeEndCentDir = {zipfile.sizeEndCentDir!r}"
assert zipfile.sizeCentralDir == 46, f"sizeCentralDir = {zipfile.sizeCentralDir!r}"
assert zipfile.sizeEndCentDir64 == 56, f"sizeEndCentDir64 = {zipfile.sizeEndCentDir64!r}"
assert zipfile.sizeEndCentDir64Locator == 20, \
    f"sizeEndCentDir64Locator = {zipfile.sizeEndCentDir64Locator!r}"

print("struct_size_constants OK")
