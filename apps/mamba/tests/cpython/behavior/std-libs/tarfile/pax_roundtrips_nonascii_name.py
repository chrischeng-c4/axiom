# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "pax_roundtrips_nonascii_name"
# subject = "tarfile.TarFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarFile: a non-ASCII member name survives a PAX-format write/read round-trip via the PAX path= record"""
import tarfile
import io

_name = "foo㍴"  # contains a non-ASCII CJK compatibility character

_buf = io.BytesIO()
with tarfile.open(fileobj=_buf, mode="w", format=tarfile.PAX_FORMAT) as _tf:
    _u = tarfile.TarInfo(_name)
    _u.size = 0
    _tf.addfile(_u)
_buf.seek(0)
with tarfile.open(fileobj=_buf, mode="r") as _tf:
    assert _tf.getnames() == [_name], f"unicode name = {_tf.getnames()!r}"

print("pax_roundtrips_nonascii_name OK")
