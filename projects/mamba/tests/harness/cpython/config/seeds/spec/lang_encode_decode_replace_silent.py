# Spec seed for CPython UnicodeDecodeError / UnicodeEncodeError /
# TypeError / LookupError contract on the encode-decode corners
# that mamba silently coerces. Surface: CPython rejects (1) decoding
# invalid utf-8 bytes — UnicodeDecodeError, not silent
# replacement-char string; (2) encoding non-ASCII to ASCII —
# UnicodeEncodeError, not silent `?` substitution; (3) `bytes(s)`
# on a `str` without an explicit encoding — TypeError ("string
# argument without an encoding"), not silent utf-8 conversion;
# (4) using a codec name that doesn't exist — LookupError ("unknown
# encoding"), not silent pass-through. The `errors='strict'` flag
# is the default and must also raise — mamba ignores it.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • str(b'\xff\xfe\xfd', 'utf-8')                 → mamba: repr-string
#                                                       (UnicodeDecodeError)
#   • b'\xff\xfe\xfd'.decode('utf-8')               → mamba: '���'
#                                                       (UnicodeDecodeError)
#   • b'\xff\xfe\xfd'.decode('utf-8',errors='strict')→ mamba: '���'
#                                                       (UnicodeDecodeError)
#   • 'café'.encode('ascii')                        → mamba: b'caf?'
#                                                       (UnicodeEncodeError)
#   • 'café'.encode('ascii',errors='strict')        → mamba: b'caf?'
#                                                       (UnicodeEncodeError)
#   • bytes('hello')                                → mamba: b'hello'
#                                                       (TypeError)
#   • 'café'.encode('nonexistent-codec')            → mamba: utf-8 bytes
#                                                       (LookupError)
#   • b'abc'.decode('nonexistent-codec')            → mamba: 'abc'
#                                                       (LookupError)
#
# CPython contract:
#   bytes.decode(encoding)
#       where bytes are invalid for the encoding
#       → UnicodeDecodeError("'utf-8' codec can't decode byte ...");
#   str.encode(encoding)
#       where str contains characters not representable in the encoding
#       → UnicodeEncodeError("'ascii' codec can't encode character ...");
#   bytes(s) where s is a str and no encoding is given
#       → TypeError("string argument without an encoding");
#   {str.encode|bytes.decode}(unknown_codec)
#       → LookupError("unknown encoding: ...").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so
# the runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_bad_utf8: Any = b'\xff\xfe\xfd'
_unicode_str: Any = "café"
_str_no_enc: Any = "hello"
_ok_bytes: Any = b'abc'

# str(bad_utf8, 'utf-8') — invalid utf-8 sequence
try:
    _ = str(_bad_utf8, 'utf-8')
    raise AssertionError("str(bad_utf8, 'utf-8') must raise UnicodeDecodeError")
except UnicodeDecodeError:
    _ledger.append(1)

# bad_utf8.decode('utf-8') — invalid utf-8 sequence
try:
    _ = _bad_utf8.decode('utf-8')
    raise AssertionError("bad_utf8.decode('utf-8') must raise UnicodeDecodeError")
except UnicodeDecodeError:
    _ledger.append(1)

# bad_utf8.decode('utf-8', errors='strict') — strict is default but must still raise
try:
    _ = _bad_utf8.decode('utf-8', errors='strict')
    raise AssertionError("bad_utf8.decode('utf-8', strict) must raise UnicodeDecodeError")
except UnicodeDecodeError:
    _ledger.append(1)

# "café".encode('ascii') — non-ASCII to ASCII
try:
    _ = _unicode_str.encode('ascii')
    raise AssertionError("'café'.encode('ascii') must raise UnicodeEncodeError")
except UnicodeEncodeError:
    _ledger.append(1)

# "café".encode('ascii', errors='strict') — strict is default but must still raise
try:
    _ = _unicode_str.encode('ascii', errors='strict')
    raise AssertionError("'café'.encode('ascii', strict) must raise UnicodeEncodeError")
except UnicodeEncodeError:
    _ledger.append(1)

# bytes("hello") — str without encoding
try:
    _ = bytes(_str_no_enc)
    raise AssertionError("bytes('hello') must raise TypeError")
except TypeError:
    _ledger.append(1)

# "café".encode('nonexistent-codec') — unknown codec
try:
    _ = _unicode_str.encode('nonexistent-codec')
    raise AssertionError("'café'.encode('nonexistent-codec') must raise LookupError")
except LookupError:
    _ledger.append(1)

# b'abc'.decode('nonexistent-codec') — unknown codec
try:
    _ = _ok_bytes.decode('nonexistent-codec')
    raise AssertionError("b'abc'.decode('nonexistent-codec') must raise LookupError")
except LookupError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_encode_decode_replace_silent {sum(_ledger)} asserts")
