# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(unicodedata, 'lookup')`
# (the documented "unicodedata exposes the lookup name->char
# helper" — mamba returns False), `hasattr(unicodedata, 'combining'
# )` (the documented "unicodedata exposes the combining helper" —
# mamba returns False), `hasattr(unicodedata, 'digit')` (the
# documented "unicodedata exposes the digit helper" — mamba returns
# False), `hasattr(unicodedata, 'numeric')` (the documented
# "unicodedata exposes the numeric helper" — mamba returns False),
# `hasattr(unicodedata, 'east_asian_width')` (the documented
# "unicodedata exposes the east_asian_width helper" — mamba
# returns False), `hasattr(unicodedata, 'mirrored')` (the
# documented "unicodedata exposes the mirrored helper" — mamba
# returns False), `unicodedata.name('A') == 'LATIN CAPITAL LETTER A
# '` (the documented "Unicode name for 'A' is its Unicode
# character database name" — mamba returns 'UNICODE CHAR 0041' — a
# synthetic placeholder format), `isinstance(unicodedata.unidata_
# version, str)` (the documented "unidata_version is a string like
# '15.0.0'" — mamba returns 'function' — attribute is a callable
# stub), `hasattr(codecs, 'CodecInfo')` (the documented "codecs
# exposes the CodecInfo namedtuple" — mamba returns False), and
# `codecs.encode('abc', 'rot13') == 'nop'` (the documented "codecs
# encode supports the rot13 string codec" — mamba returns b'abc' —
# rot13 codec missing, returns input unchanged).
# Ten-pack pinned to atomic 292.
#
# Behavioral edges that CONFORM on mamba (unicodedata — hasattr
# name/category/bidirectional/decimal/normalize/unidata_version +
# category 'Lu'/'Nd'/'Zs' + decimal('5')==5 + normalize. codecs —
# hasattr encode/decode/lookup/getencoder/getdecoder/getreader/
# getwriter/BOM_UTF8/BOM_UTF16/BOM_UTF16_BE/BOM_UTF16_LE/BOM_UTF32/
# register/open + encode/decode utf-8 + BOM_UTF8 triplet.
# mimetypes — hasattr guess_type/guess_all_extensions/guess_
# extension/add_type/init/knownfiles/suffix_map/encodings_map/
# types_map/MimeTypes + guess_type/types_map/guess_extension.
# quopri — hasattr encodestring/decodestring/encode/decode +
# encode/decode round-trip + '=' escaping) are covered in the
# matching pass fixture `test_codecs_mimetypes_quopri_unicodedata_
# value_ops`.
import unicodedata
import codecs


_ledger: list[int] = []

# 1) hasattr(unicodedata, 'lookup') — lookup name->char helper
#    (mamba: returns False)
assert hasattr(unicodedata, "lookup") == True; _ledger.append(1)

# 2) hasattr(unicodedata, 'combining') — combining-class helper
#    (mamba: returns False)
assert hasattr(unicodedata, "combining") == True; _ledger.append(1)

# 3) hasattr(unicodedata, 'digit') — digit-value helper
#    (mamba: returns False)
assert hasattr(unicodedata, "digit") == True; _ledger.append(1)

# 4) hasattr(unicodedata, 'numeric') — numeric-value helper
#    (mamba: returns False)
assert hasattr(unicodedata, "numeric") == True; _ledger.append(1)

# 5) hasattr(unicodedata, 'east_asian_width') — east-asian-width helper
#    (mamba: returns False)
assert hasattr(unicodedata, "east_asian_width") == True; _ledger.append(1)

# 6) hasattr(unicodedata, 'mirrored') — mirrored-flag helper
#    (mamba: returns False)
assert hasattr(unicodedata, "mirrored") == True; _ledger.append(1)

# 7) unicodedata.name('A') == 'LATIN CAPITAL LETTER A' — UCD canonical name
#    (mamba: returns 'UNICODE CHAR 0041' — synthetic placeholder)
assert unicodedata.name("A") == "LATIN CAPITAL LETTER A"; _ledger.append(1)

# 8) isinstance(unicodedata.unidata_version, str) — UCD version string
#    (mamba: attribute is a callable stub, type 'function')
assert isinstance(unicodedata.unidata_version, str) == True; _ledger.append(1)

# 9) hasattr(codecs, 'CodecInfo') — CodecInfo namedtuple
#    (mamba: returns False)
assert hasattr(codecs, "CodecInfo") == True; _ledger.append(1)

# 10) codecs.encode('abc', 'rot13') == 'nop' — rot13 string codec
#     (mamba: returns b'abc' — rot13 codec missing, returns unchanged)
assert codecs.encode("abc", "rot13") == "nop"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_unicodedata_codecs_silent {sum(_ledger)} asserts")
