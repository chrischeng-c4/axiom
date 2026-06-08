# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "utf8_sig_stream_strips_bom_any_chunk"
# subject = "codecs.getreader"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.getreader: the utf-8-sig StreamReader strips the BOM regardless of read chunk size: reading 'ABC¡∀XYZ' (BOM-prefixed) in size hints None/1/2/7/64 all reproduce the original string"""
import codecs

import io
_unistring = "ABC¡∀XYZ"
_bytestring = codecs.BOM_UTF8 + b"ABC\xc2\xa1\xe2\x88\x80XYZ"
for _sizehint in (None, 1, 2, 7, 64):
    _istream = codecs.getreader("utf-8-sig")(io.BytesIO(_bytestring))
    _ostream = io.StringIO()
    while True:
        _data = _istream.read() if _sizehint is None else _istream.read(_sizehint)
        if not _data:
            break
        _ostream.write(_data)
    assert _ostream.getvalue() == _unistring, f"utf-8-sig sizehint={_sizehint}"

print("utf8_sig_stream_strips_bom_any_chunk OK")
