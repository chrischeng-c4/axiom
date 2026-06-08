# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "itn_nti_roundtrip_gnu_range"
# subject = "tarfile.itn"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.itn: itn and nti are inverses across the GNU range: nti(itn(n, GNU_FORMAT)) == n for representative n including 0, 2**21-1, 2**21, 2**32-1, -1, -100"""
import tarfile

for n in (0, 1, 2097151, 2097152, 4294967295, -1, -100):
    enc = tarfile.itn(n, format=tarfile.GNU_FORMAT)
    assert tarfile.nti(enc) == n, f"round-trip {n} -> {enc!r}"

print("itn_nti_roundtrip_gnu_range OK")
