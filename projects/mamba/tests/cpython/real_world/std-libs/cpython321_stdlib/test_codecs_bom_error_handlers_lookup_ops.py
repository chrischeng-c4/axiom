# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_codecs_bom_error_handlers_lookup_ops"
# subject = "cpython321.test_codecs_bom_error_handlers_lookup_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_codecs_bom_error_handlers_lookup_ops.py"
# status = "filled"
# ///
"""cpython321.test_codecs_bom_error_handlers_lookup_ops: execute CPython 3.12 seed test_codecs_bom_error_handlers_lookup_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `codecs` module's BOM
# constants / error-handler callable surface / getencoder + getdecoder
# factory / module-level attribute discipline — the build-time
# constants every UTF-aware reader uses to detect/strip a BOM
# (`BOM_UTF8/16/32` family), the named error-handler callables every
# robust decoder reaches through (`strict_errors`, `ignore_errors`,
# `replace_errors`, `xmlcharrefreplace_errors`, `backslashreplace_errors`),
# and the function-factory entry points that every codecs-driven
# encoding pipeline calls (`getencoder` / `getdecoder` returning a
# callable). Existing `test_codecs.py` covers `BOM_UTF8`,
# `BOM_UTF16_LE`, `BOM_UTF16_BE` and roundtrip encode for utf-8/ascii/
# latin-1; this seed fills the BOM_UTF16 / BOM_UTF32 / BOM_UTF32_LE /
# BOM_UTF32_BE constants + the error-handler callables + the
# `getencoder` / `getdecoder` factories — the matching subset that
# neither existing seed covers.
#
# Surface (the matching subset between mamba and CPython):
#   • codecs.BOM_UTF8 == b'\xef\xbb\xbf'  (covered upstream, asserted
#     again for length/bytes-type discipline);
#   • codecs.BOM_UTF16    == b'\xff\xfe'      (system-endian BOM,
#                                                identical to LE on LE);
#   • codecs.BOM_UTF16_LE == b'\xff\xfe';
#   • codecs.BOM_UTF16_BE == b'\xfe\xff';
#   • codecs.BOM_UTF32    == b'\xff\xfe\x00\x00';
#   • codecs.BOM_UTF32_LE == b'\xff\xfe\x00\x00';
#   • codecs.BOM_UTF32_BE == b'\x00\x00\xfe\xff';
#   • BOM constants are `bytes`;
#   • length discipline: BOM_UTF8 == 3, BOM_UTF16_LE/BE == 2,
#     BOM_UTF32_LE/BE == 4;
#   • codecs.strict_errors / ignore_errors / replace_errors /
#     xmlcharrefreplace_errors / backslashreplace_errors are
#     all callable named functions;
#   • codecs.getencoder('utf-8') / getencoder('ascii') /
#     getencoder('latin-1') return a callable;
#   • codecs.getdecoder('utf-8') / getdecoder('ascii') /
#     getdecoder('latin-1') return a callable;
#   • module attribute discipline — every name above is present on
#     the module.
import codecs
_ledger: list[int] = []

# BOM_UTF8 — 3-byte UTF-8 marker
assert codecs.BOM_UTF8 == b'\xef\xbb\xbf'; _ledger.append(1)
assert isinstance(codecs.BOM_UTF8, bytes); _ledger.append(1)
assert len(codecs.BOM_UTF8) == 3; _ledger.append(1)

# BOM_UTF16 — system-endian (LE on little-endian hosts)
assert codecs.BOM_UTF16 == b'\xff\xfe'; _ledger.append(1)
assert isinstance(codecs.BOM_UTF16, bytes); _ledger.append(1)
assert len(codecs.BOM_UTF16) == 2; _ledger.append(1)

# BOM_UTF16_LE
assert codecs.BOM_UTF16_LE == b'\xff\xfe'; _ledger.append(1)
assert isinstance(codecs.BOM_UTF16_LE, bytes); _ledger.append(1)
assert len(codecs.BOM_UTF16_LE) == 2; _ledger.append(1)

# BOM_UTF16_BE
assert codecs.BOM_UTF16_BE == b'\xfe\xff'; _ledger.append(1)
assert isinstance(codecs.BOM_UTF16_BE, bytes); _ledger.append(1)
assert len(codecs.BOM_UTF16_BE) == 2; _ledger.append(1)

# BOM_UTF16 LE/BE pair is reverse byte order
assert codecs.BOM_UTF16_LE[0] == codecs.BOM_UTF16_BE[1]; _ledger.append(1)
assert codecs.BOM_UTF16_LE[1] == codecs.BOM_UTF16_BE[0]; _ledger.append(1)

# BOM_UTF32 — system-endian (LE on little-endian hosts)
assert codecs.BOM_UTF32 == b'\xff\xfe\x00\x00'; _ledger.append(1)
assert isinstance(codecs.BOM_UTF32, bytes); _ledger.append(1)
assert len(codecs.BOM_UTF32) == 4; _ledger.append(1)

# BOM_UTF32_LE
assert codecs.BOM_UTF32_LE == b'\xff\xfe\x00\x00'; _ledger.append(1)
assert isinstance(codecs.BOM_UTF32_LE, bytes); _ledger.append(1)
assert len(codecs.BOM_UTF32_LE) == 4; _ledger.append(1)

# BOM_UTF32_BE
assert codecs.BOM_UTF32_BE == b'\x00\x00\xfe\xff'; _ledger.append(1)
assert isinstance(codecs.BOM_UTF32_BE, bytes); _ledger.append(1)
assert len(codecs.BOM_UTF32_BE) == 4; _ledger.append(1)

# BOM_UTF32 LE/BE are reverse byte order
assert codecs.BOM_UTF32_LE[0] == codecs.BOM_UTF32_BE[3]; _ledger.append(1)
assert codecs.BOM_UTF32_LE[1] == codecs.BOM_UTF32_BE[2]; _ledger.append(1)
assert codecs.BOM_UTF32_LE[2] == codecs.BOM_UTF32_BE[1]; _ledger.append(1)
assert codecs.BOM_UTF32_LE[3] == codecs.BOM_UTF32_BE[0]; _ledger.append(1)

# All BOMs are bytes, never str
for _bom in (codecs.BOM_UTF8, codecs.BOM_UTF16, codecs.BOM_UTF16_LE,
             codecs.BOM_UTF16_BE, codecs.BOM_UTF32, codecs.BOM_UTF32_LE,
             codecs.BOM_UTF32_BE):
    assert isinstance(_bom, bytes); _ledger.append(1)

# BOM lengths are non-zero and bounded
for _bom in (codecs.BOM_UTF8, codecs.BOM_UTF16_LE, codecs.BOM_UTF16_BE,
             codecs.BOM_UTF32_LE, codecs.BOM_UTF32_BE):
    assert 0 < len(_bom) <= 4; _ledger.append(1)

# error handlers — all callable
assert callable(codecs.strict_errors); _ledger.append(1)
assert callable(codecs.ignore_errors); _ledger.append(1)
assert callable(codecs.replace_errors); _ledger.append(1)
assert callable(codecs.xmlcharrefreplace_errors); _ledger.append(1)
assert callable(codecs.backslashreplace_errors); _ledger.append(1)

# getencoder — returns a callable for known codecs
assert callable(codecs.getencoder("utf-8")); _ledger.append(1)
assert callable(codecs.getencoder("ascii")); _ledger.append(1)
assert callable(codecs.getencoder("latin-1")); _ledger.append(1)

# getdecoder — returns a callable for known codecs
assert callable(codecs.getdecoder("utf-8")); _ledger.append(1)
assert callable(codecs.getdecoder("ascii")); _ledger.append(1)
assert callable(codecs.getdecoder("latin-1")); _ledger.append(1)

# Module-level attribute discipline — BOM constants
for _name in ('BOM_UTF8', 'BOM_UTF16', 'BOM_UTF16_LE', 'BOM_UTF16_BE',
              'BOM_UTF32', 'BOM_UTF32_LE', 'BOM_UTF32_BE'):
    assert hasattr(codecs, _name); _ledger.append(1)

# Module-level attribute discipline — error handlers
for _name in ('strict_errors', 'ignore_errors', 'replace_errors',
              'xmlcharrefreplace_errors', 'backslashreplace_errors'):
    assert hasattr(codecs, _name); _ledger.append(1)

# Module-level attribute discipline — factory functions
for _name in ('getencoder', 'getdecoder', 'encode', 'decode', 'lookup'):
    assert hasattr(codecs, _name); _ledger.append(1)
    assert callable(getattr(codecs, _name)); _ledger.append(1)

# Module name discipline
assert codecs.__name__ == 'codecs'; _ledger.append(1)

# encode/decode wired through to known codecs (matching subset)
assert codecs.encode("abc", "utf-8") == b'abc'; _ledger.append(1)
assert codecs.encode("abc", "ascii") == b'abc'; _ledger.append(1)
assert codecs.encode("abc", "latin-1") == b'abc'; _ledger.append(1)
assert codecs.decode(b"abc", "utf-8") == 'abc'; _ledger.append(1)
assert codecs.decode(b"abc", "ascii") == 'abc'; _ledger.append(1)
assert codecs.decode(b"abc", "latin-1") == 'abc'; _ledger.append(1)

# encode default to utf-8
assert codecs.encode("abc") == b'abc'; _ledger.append(1)
# decode default to utf-8
assert codecs.decode(b"abc") == 'abc'; _ledger.append(1)

# BOM_UTF8 prefix is detectable by slicing
_with_bom = codecs.BOM_UTF8 + b'hello'
assert _with_bom[:3] == codecs.BOM_UTF8; _ledger.append(1)
assert _with_bom[3:] == b'hello'; _ledger.append(1)

# BOM_UTF16_LE prefix detection
_u16 = codecs.BOM_UTF16_LE + b'a\x00b\x00'
assert _u16[:2] == codecs.BOM_UTF16_LE; _ledger.append(1)
assert _u16[2:] == b'a\x00b\x00'; _ledger.append(1)

# Distinct values across the BOM family
_bom_set = {bytes(codecs.BOM_UTF8), bytes(codecs.BOM_UTF16_LE),
            bytes(codecs.BOM_UTF16_BE), bytes(codecs.BOM_UTF32_LE),
            bytes(codecs.BOM_UTF32_BE)}
assert len(_bom_set) == 5; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_codecs_bom_error_handlers_lookup_ops {sum(_ledger)} asserts")
