"""Behavior contract for third-party idna package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import idna  # type: ignore[import]

# Rule 1: ASCII domain name encodes as-is
_enc1 = idna.encode("example.com")
assert _enc1 == b"example.com", f"ascii encode = {_enc1!r}"
_dec1 = idna.decode(b"example.com")
assert _dec1 == "example.com", f"ascii decode = {_dec1!r}"

# Rule 2: Unicode label encodes to ACE (punycode) form
_enc2 = idna.encode("münchen.de")
assert isinstance(_enc2, bytes), f"encode type = {type(_enc2)!r}"
assert _enc2 == b"xn--mnchen-3ya.de", f"punycode encode = {_enc2!r}"

# Rule 3: ACE label decodes to unicode
_dec3 = idna.decode(b"xn--mnchen-3ya.de")
assert _dec3 == "münchen.de", f"punycode decode = {_dec3!r}"

# Rule 4: encode/decode round-trip preserves domain
_cases4 = ["example.com", "münchen.de", "日本.jp"]
for _domain in _cases4:
    _encoded = idna.encode(_domain)
    _decoded = idna.decode(_encoded)
    assert _decoded == _domain, f"round-trip {_domain!r} → {_decoded!r}"

# Rule 5: alabel returns ACE label for a single label (no dots)
_al5 = idna.alabel("münchen")
assert isinstance(_al5, bytes), f"alabel type = {type(_al5)!r}"
assert _al5 == b"xn--mnchen-3ya", f"alabel = {_al5!r}"

# Rule 6: alabel on ASCII label returns it as-is (bytes)
_al6 = idna.alabel("example")
assert _al6 == b"example", f"ascii alabel = {_al6!r}"

# Rule 7: ulabel decodes ACE to unicode label
_ul7 = idna.ulabel(b"xn--mnchen-3ya")
assert isinstance(_ul7, str), f"ulabel type = {type(_ul7)!r}"
assert _ul7 == "münchen", f"ulabel = {_ul7!r}"

# Rule 8: Module attributes are identity-stable across repeated reads
_enc_ref = idna.encode
_dec_ref = idna.decode
_err_ref = idna.IDNAError
_al_ref = idna.alabel
for _ in range(10):
    assert idna.encode is _enc_ref, "encode identity"
    assert idna.decode is _dec_ref, "decode identity"
    assert idna.IDNAError is _err_ref, "IDNAError identity"
    assert idna.alabel is _al_ref, "alabel identity"

print("behavior OK")
