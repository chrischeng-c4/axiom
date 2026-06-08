# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "pax_has_no_practical_field_limits"
# subject = "tarfile.TarInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarInfo: PAX_FORMAT carries oversized names and a 2**56 uid in extended headers, so tobuf(PAX_FORMAT) raises nothing where USTAR/GNU would"""
import tarfile

# PAX extended headers carry oversized values; nothing raises.
tarfile.TarInfo("123/" * 126 + "longname").tobuf(tarfile.PAX_FORMAT)

_ti = tarfile.TarInfo("name")
_ti.uid = 72057594037927936  # 2**56, overflows GNU base-256 octal/uid field
_ti.tobuf(tarfile.PAX_FORMAT)

print("pax_has_no_practical_field_limits OK")
