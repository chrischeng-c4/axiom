# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "pax_roundtrips_oversized_numeric_fields"
# subject = "tarfile.TarFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarFile: a PAX-format archive round-trips uid and mtime values too large for the classic ustar octal fields (uid=2**24, mtime=10**10), recovering the exact integers and payload"""
import tarfile
import io

_buf = io.BytesIO()
with tarfile.open(fileobj=_buf, mode="w", format=tarfile.PAX_FORMAT) as _tf:
    _big = tarfile.TarInfo("big.bin")
    _big.size = 5
    _big.uid = 16777216  # 8**8, overflows the 7-digit octal uid field
    _big.mtime = 10**10
    _tf.addfile(_big, io.BytesIO(b"hello"))
_buf.seek(0)
with tarfile.open(fileobj=_buf, mode="r") as _tf:
    _m = _tf.getmember("big.bin")
    assert _m.uid == 16777216, f"pax uid = {_m.uid!r}"
    assert _m.mtime == 10**10, f"pax mtime = {_m.mtime!r}"
    assert _tf.extractfile("big.bin").read() == b"hello", "pax data"

print("pax_roundtrips_oversized_numeric_fields OK")
