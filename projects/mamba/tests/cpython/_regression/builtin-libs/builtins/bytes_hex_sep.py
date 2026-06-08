# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# `bytes.hex(sep=None, bytes_per_sep=1)` — Python 3.8+ added two optional
# arguments to format hex with a separator and group size:
#  - `sep`: a single-character str (or single-byte bytes); inserted between
#    groups. None → no separator.
#  - `bytes_per_sep`: positive groups bytes from the right end (the leftmost
#    group may be shorter); negative groups from the left end. Default 1.
#
# Mamba's `mb_bytes_hex` ignored both arguments — `b'Hello'.hex(' ')` came
# back as `'48656c6c6f'` instead of `'48 65 6c 6c 6f'`.
#
# Fix in `runtime/bytes_ops.rs::mb_bytes_hex_with_sep`: parse `sep` from
# either Str (first char) or Bytes (first byte); thread `bytes_per_sep`
# through; group from the right when positive, from the left when negative.
# Empty bytes / sep=None / group=0 fall back to the unseparated form.

# No separator (regression baseline).
print(b'Hello'.hex())                          # '48656c6c6f'

# 1-char str sep, default bytes_per_sep=1.
print(b'Hello'.hex(' '))                       # '48 65 6c 6c 6f'
print(b'Hello'.hex(':'))                       # '48:65:6c:6c:6f'

# Group from the right (positive bytes_per_sep): leftmost group may be smaller.
print(b'\xb9\x01\xef'.hex(':', 2))              # 'b9:01ef'
print(b'\x01\x02\x03\x04\x05'.hex('_', 2))      # '01_0203_0405'

# Group from the left (negative bytes_per_sep): rightmost group may be smaller.
print(b'\xb9\x01\xef'.hex(':', -2))             # 'b901:ef'
print(b'\x01\x02\x03\x04\x05'.hex('_', -2))     # '0102_0304_05'

# Even-length input: positive and negative produce the same grouping.
print(b'\x01\x02\x03\x04'.hex('_', 2))          # '0102_0304'
print(b'\x01\x02\x03\x04'.hex('_', -2))         # '0102_0304'

# Edge: empty bytes.
print(repr(b''.hex(' ')))                      # ''
