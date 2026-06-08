# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# `bytes.decode(encoding, errors)` ignored both arguments — `mb_bytes_decode`
# always ran `String::from_utf8_lossy` regardless of the requested encoding.
# So `b'h\xe9llo'.decode('latin-1')` returned `h�llo` (UTF-8 replacement
# of the invalid UTF-8 byte) instead of `héllo`, and `decode('utf-8',
# errors='ignore')` still emitted `�` characters because the errors
# kwarg was silently dropped.
#
# Fix in `runtime/bytes_ops.rs`:
#   - Add `mb_bytes_decode_with(b, encoding, errors)` covering utf-8 /
#     ascii / latin-1 with `strict` / `ignore` / `replace`.
#   - Hand-rolled `decode_utf8` so we control the per-error step
#     (`ignore` drops bad bytes, `replace` / `strict` emit one U+FFFD
#     per invalid sequence — matching CPython byte-by-byte progress).
#   - Update the `"decode"` dispatch arm to accept positional
#     `(encoding[, errors])` AND a trailing kwargs Dict pulling
#     `encoding=` / `errors=`.

# Default — UTF-8 (regression).
print(b'h\xc3\xa9llo'.decode())                          # héllo
print(b'\xc3\xa9'.decode())                              # é
print(b'plain'.decode())                                 # plain

# Explicit utf-8.
print(b'h\xc3\xa9llo'.decode('utf-8'))                   # héllo
print(b'plain'.decode('utf-8'))                          # plain

# Latin-1 round-trip — single-byte codepoint, never errors.
print(b'h\xe9llo'.decode('latin-1'))                     # héllo
print(b'caf\xe9'.decode('iso-8859-1'))                   # café
print(b'\x80\x81'.decode(encoding='latin-1'))            # \x80\x81 (control chars)

# UTF-8 errors — `ignore` drops bad bytes, `replace` substitutes U+FFFD.
print(b'h\xff\xfe'.decode('utf-8', errors='ignore'))     # h
print(b'h\xff\xfe'.decode('utf-8', errors='replace'))    # h��
print(b'h\xff\xfe'.decode('utf-8', 'ignore'))            # h  (positional)
print(b'h\xff\xfe'.decode('utf-8', 'replace'))           # h��

# ASCII errors.
print(b'plain'.decode('ascii'))                          # plain
print(b'\x80'.decode('ascii', errors='ignore'))          # ''
print(b'\x80'.decode('ascii', errors='replace'))         # �
print(b'A\x80B'.decode('ascii', 'ignore'))               # AB

# Mixed valid + invalid surrounding context.
print(b'\xe2\x82\xac\xff'.decode('utf-8', 'ignore'))     # €
print(b'\xff\xe2\x82\xac'.decode('utf-8', 'replace'))    # �€

# bytearray decode follows the same path.
print(bytearray(b'h\xe9llo').decode('latin-1'))          # héllo
