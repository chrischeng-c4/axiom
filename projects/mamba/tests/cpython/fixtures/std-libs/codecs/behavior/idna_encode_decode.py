# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "idna_encode_decode"
# subject = "str.encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""str.encode: idna applies ToASCII/ToUnicode: 'pythön.org'.encode('idna') is b'xn--pythn-mua.org' and decodes back, while pure-ASCII labels pass through unchanged"""
import codecs

# ASCII labels pass through; non-ASCII labels get the xn-- prefix.
assert "python.org".encode("idna") == b"python.org"
assert "python.org.".encode("idna") == b"python.org."
assert "pythön.org".encode("idna") == b"xn--pythn-mua.org"
assert "pythön.org.".encode("idna") == b"xn--pythn-mua.org."
# decode is the inverse of encode.
assert b"xn--pythn-mua.org".decode("idna") == "pythön.org"
assert b"python.org".decode("idna") == "python.org"

print("idna_encode_decode OK")
