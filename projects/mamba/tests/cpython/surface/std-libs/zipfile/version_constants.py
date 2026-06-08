# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "surface"
# case = "version_constants"
# subject = "zipfile.DEFAULT_VERSION"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.DEFAULT_VERSION: DEFAULT_VERSION == 20 and ZIP64_VERSION == 45 (advertised PKZIP feature-version constants)"""
import zipfile

assert zipfile.DEFAULT_VERSION == 20, f"DEFAULT_VERSION = {zipfile.DEFAULT_VERSION!r}"
assert zipfile.ZIP64_VERSION == 45, f"ZIP64_VERSION = {zipfile.ZIP64_VERSION!r}"

print("version_constants OK")
