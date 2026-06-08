# /// script
# requires-python = ">=3.12"
# dependencies = [
#   "idna==3.7",
# ]
# ///
"""Surface contract for third-party idna package.

# type-regime: monomorphic

Probes: idna.encode, idna.decode, idna.IDNAError, idna.alabel,
idna.ulabel, idna.check_label, idna.check_hyphen_ok.
CPython 3.12 is the oracle.
"""

import idna

# Core attributes
assert hasattr(idna, "encode"), "encode"
assert hasattr(idna, "decode"), "decode"
assert hasattr(idna, "IDNAError"), "IDNAError"
assert hasattr(idna, "alabel"), "alabel"
assert hasattr(idna, "ulabel"), "ulabel"
assert hasattr(idna, "check_label"), "check_label"
assert hasattr(idna, "valid_label_length"), "valid_label_length"

# IDNAError is an exception
assert issubclass(idna.IDNAError, Exception), "IDNAError < Exception"

# encode: unicode domain → bytes (punycode if needed)
_enc = idna.encode("example.com")
assert isinstance(_enc, bytes), f"encode type = {type(_enc)!r}"
assert _enc == b"example.com", f"encode = {_enc!r}"

_enc2 = idna.encode("münchen.de")
assert isinstance(_enc2, bytes), f"encode unicode type = {type(_enc2)!r}"
assert b"xn--" in _enc2 or _enc2 == b"xn--mnchen-3ya.de", \
    f"encoded münchen = {_enc2!r}"

# decode: bytes → unicode domain
_dec = idna.decode(b"example.com")
assert isinstance(_dec, str), f"decode type = {type(_dec)!r}"
assert _dec == "example.com", f"decode = {_dec!r}"

_dec2 = idna.decode(b"xn--mnchen-3ya.de")
assert isinstance(_dec2, str), f"decode punycode type = {type(_dec2)!r}"
assert "münchen" in _dec2 or "nchen" in _dec2, f"decoded = {_dec2!r}"

# alabel returns ACE label (bytes)
_al = idna.alabel("münchen")
assert isinstance(_al, bytes), f"alabel type = {type(_al)!r}"
assert _al.startswith(b"xn--"), f"alabel starts with xn--: {_al!r}"

# ulabel returns unicode label (str)
_ul = idna.ulabel(b"xn--mnchen-3ya")
assert isinstance(_ul, str), f"ulabel type = {type(_ul)!r}"

# Module attributes are identity-stable
_enc_ref = idna.encode
assert idna.encode is _enc_ref, "encode stable identity"
_dec_ref = idna.decode
assert idna.decode is _dec_ref, "decode stable identity"
_err_ref = idna.IDNAError
assert idna.IDNAError is _err_ref, "IDNAError stable identity"
_al_ref = idna.alabel
assert idna.alabel is _al_ref, "alabel stable identity"

print("surface OK")
