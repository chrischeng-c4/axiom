# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "encodedfile_recodes_read_and_write"
# subject = "codecs.EncodedFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.EncodedFile: EncodedFile recodes between a file encoding and a data encoding: reading 'ü' (latin-1 on disk) as utf-8 and writing utf-8 'ü' lands latin-1 b'\\xfc' on disk; the wrapped file closes on context exit"""
import codecs

import io
# Reading recodes the file encoding to the data encoding; the base file closes.
_f = io.BytesIO(b"\xc3\xbc")  # 'ü' as utf-8
with codecs.EncodedFile(_f, "latin-1", "utf-8") as _ef:
    assert _ef.read() == b"\xfc", "utf-8 bytes recoded to latin-1"
assert _f.closed, "EncodedFile closes the wrapped file on exit"
# Writing recodes the data encoding to the file encoding.
_out = io.BytesIO()
_ef2 = codecs.EncodedFile(_out, "utf-8", "latin-1")
_ef2.write(b"\xc3\xbc")
assert _out.getvalue() == b"\xfc", f"recoded write = {_out.getvalue()!r}"

print("encodedfile_recodes_read_and_write OK")
