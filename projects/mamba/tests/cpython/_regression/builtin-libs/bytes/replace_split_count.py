# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# `bytes.replace` ignored its `count` argument and `bytes.split` ignored
# `maxsplit` — both methods always operated in their unbounded form. So:
#
#   b"aaaa".replace(b"a", b"b", 2)        # → b"bbbb"  (wrong; should be b"bbaa")
#   b"a,b,c,d".split(b",", 2)             # → [b"a", b"b", b"c", b"d"]  (wrong)
#
# Fix in `runtime/bytes_ops.rs`:
#   - Add `mb_bytes_replace_count(haystack, old, new, count)`. `count < 0`
#     (or omitted) means "unbounded"; `count == 0` is a no-op; for empty
#     `old`, the natural insertion count is `len(haystack) + 1` and the
#     count cap applies before that ceiling (CPython rule).
#   - Add `mb_bytes_split_max(haystack, sep, maxsplit)` covering both
#     the explicit-sep path and the whitespace path. Once `maxsplit`
#     splits have been emitted, the rest of the buffer is appended
#     verbatim as the last element (whitespace path strips the leading
#     run on that remainder, matching CPython).
#   - Bare `mb_bytes_replace` / `mb_bytes_split` forward through the
#     `_count` / `_max` versions with `MbValue::none()` to stay
#     byte-equivalent at existing call sites.

# replace with count
print(b"aaaa".replace(b"a", b"b", 2))           # b'bbaa'
print(b"aaaa".replace(b"a", b"b", 0))           # b'aaaa'  (count=0 → no-op)
print(b"aaaa".replace(b"a", b"b", -1))          # b'bbbb'  (negative → unbounded)
print(b"aaaa".replace(b"a", b"b"))              # b'bbbb'  (default → unbounded)
print(b"aaaa".replace(b"a", b"b", 100))         # b'bbbb'  (cap above len)

# replace with empty needle (insertion at every gap)
print(b"aaaa".replace(b"", b"-", 2))            # b'-a-aaa'
print(b"aaaa".replace(b"", b"-"))               # b'-a-a-a-a-'  (len+1 inserts)
print(b"aaaa".replace(b"", b"-", 0))            # b'aaaa'

# replace where old is multi-byte
print(b"abcabcabc".replace(b"abc", b"X", 2))    # b'XXabc'
print(b"abcabcabc".replace(b"abc", b"X"))       # b'XXX'

# split with maxsplit, explicit sep
print(b"a,b,c,d".split(b",", 2))                # [b'a', b'b', b'c,d']
print(b"a,b,c,d".split(b",", 0))                # [b'a,b,c,d']
print(b"a,b,c,d".split(b",", -1))               # [b'a', b'b', b'c', b'd']
print(b"a,b,c,d".split(b","))                   # [b'a', b'b', b'c', b'd']
print(b"a,b,c,d".split(b",", 100))              # [b'a', b'b', b'c', b'd']

# split with maxsplit, whitespace path
print(b"  a  b   c   ".split(None, 1))          # [b'a', b'b   c   ']
print(b"  a  b   c   ".split(None, 2))          # [b'a', b'b', b'c   ']
print(b"  a  b   c   ".split(None))             # [b'a', b'b', b'c']
print(b"  a  b   c   ".split())                 # [b'a', b'b', b'c']
print(b"  a  b   c   ".split(None, 0))          # [b'a  b   c   ']

# bytearray shares the dispatcher — same count/maxsplit semantics.
# (Note: mamba currently returns plain `bytes` from these methods rather
# than preserving `bytearray`; that's a separate gap, so this fixture
# only checks count/maxsplit behaviour.)
ba = bytearray(b"aaaa")
print(bytes(ba.replace(b"a", b"b", 2)))         # b'bbaa'
ba2 = bytearray(b"a,b,c,d")
print([bytes(p) for p in ba2.split(b",", 2)])   # [b'a', b'b', b'c,d']
