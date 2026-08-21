# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "ustar_name_within_limits_ok"
# subject = "tarfile.TarInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarInfo: USTAR accepts a 100-char name and a name that splits into prefix(<=155)+name(<=100); tobuf(USTAR_FORMAT) succeeds for both"""
import tarfile

# A 100-char name fits within the USTAR name field.
tarfile.TarInfo("0123456789" * 10).tobuf(tarfile.USTAR_FORMAT)

# A name splittable into prefix(<=155) + name(<=100) is fine.
tarfile.TarInfo("123/" * 62 + "longname").tobuf(tarfile.USTAR_FORMAT)

print("ustar_name_within_limits_ok OK")
