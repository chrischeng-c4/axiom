# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "zipinfo_repr"
# subject = "zipfile.ZipInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zipfile"
# status = "filled"
# ///
"""zipfile.ZipInfo: repr(ZipInfo(filename='empty')) is the stable string <ZipInfo filename='empty' file_size=0>"""
import zipfile

assert repr(zipfile.ZipInfo(filename="empty")) == "<ZipInfo filename='empty' file_size=0>", \
    "empty ZipInfo repr"

print("zipinfo_repr OK")
