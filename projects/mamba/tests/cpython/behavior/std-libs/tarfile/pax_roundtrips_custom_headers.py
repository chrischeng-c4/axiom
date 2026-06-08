# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "pax_roundtrips_custom_headers"
# subject = "tarfile.TarFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarFile: a custom pax_headers key/value pair round-trips by key through a PAX-format archive"""
import tarfile
import io

_buf = io.BytesIO()
with tarfile.open(fileobj=_buf, mode="w", format=tarfile.PAX_FORMAT) as _tf:
    _x = tarfile.TarInfo("x")
    _x.size = 0
    _x.pax_headers["VENDOR.note"] = "hi"
    _tf.addfile(_x)
_buf.seek(0)
with tarfile.open(fileobj=_buf, mode="r") as _tf:
    _m = _tf.getmember("x")
    assert _m.pax_headers.get("VENDOR.note") == "hi", f"pax meta = {_m.pax_headers!r}"

print("pax_roundtrips_custom_headers OK")
