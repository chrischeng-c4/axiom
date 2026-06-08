# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# `str.encode(encoding, errors)` ignored its arguments entirely:
#
#   "héllo".encode('ascii', errors='ignore')   # → b'h\xc3\xa9llo'  (wrong)
#
# Mamba's dispatcher routed `encode` to a 0-arg `mb_str_encode` that
# always emits UTF-8, so encoding/errors were silently dropped. CPython
# accepts both positional and keyword forms — `("ascii", "ignore")`,
# `("ascii", errors="replace")`, `(encoding="latin-1")`, etc.
#
# Fix in `runtime/string_ops.rs`:
#   - Add `mb_str_encode_with(s, encoding, errors)` covering utf-8,
#     ascii, and latin-1 with `strict` / `ignore` / `replace`.
#   - Update the `"encode"` dispatch arm to (a) inspect a trailing
#     kwargs Dict for `encoding=` / `errors=`, (b) fall back to the
#     no-arg path when both are absent, (c) otherwise route through
#     the new helper.
#
# Note: strict-mode UnicodeEncodeError isn't modeled yet; we emit `?`
# rather than raising so a bare `s.encode("ascii")` on non-ASCII text
# stays well-defined. This is a coverage gap to revisit when CPython
# exception wiring lands for the codec path.

# Default — UTF-8.
print("hi".encode())                                # b'hi'
print("héllo".encode())                             # b'h\xc3\xa9llo'

# Explicit utf-8.
print("hi".encode("utf-8"))                         # b'hi'
print("héllo".encode("utf-8"))                      # b'h\xc3\xa9llo'

# ASCII with ignore — drops non-ASCII chars.
print("héllo".encode("ascii", errors="ignore"))     # b'hllo'
print("héllo".encode("ascii", "ignore"))            # b'hllo'  (positional)
print("café".encode(encoding="ascii", errors="ignore"))  # b'caf'

# ASCII with replace — substitutes `?`.
print("héllo".encode("ascii", errors="replace"))    # b'h?llo'
print("héllo".encode("ascii", "replace"))           # b'h?llo'

# Latin-1 — round-trip of ≤U+00FF as a single byte.
print("héllo".encode("latin-1"))                    # b'h\xe9llo'
print("café".encode("iso-8859-1"))                  # b'caf\xe9'

# Latin-1 errors path.
print("π".encode("latin-1", errors="ignore"))       # b''
print("π".encode("latin-1", errors="replace"))      # b'?'

# All-ASCII content survives any of the above.
print("plain".encode("ascii"))                      # b'plain'
print("plain".encode("ascii", "strict"))            # b'plain'
