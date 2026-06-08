# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "gnu_lifts_name_limits"
# subject = "tarfile.TarInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarInfo: GNU_FORMAT encodes long names and linknames that overflow USTAR, so tobuf(GNU_FORMAT) succeeds where USTAR would raise"""
import tarfile

# GNU encodes long names, so an overlong name now succeeds.
tarfile.TarInfo("123/" * 126 + "longname").tobuf(tarfile.GNU_FORMAT)

# GNU also encodes long linknames.
_ti = tarfile.TarInfo("longlink")
_ti.linkname = "123/" * 126 + "longname"
_ti.tobuf(tarfile.GNU_FORMAT)

print("gnu_lifts_name_limits OK")
