# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# `bytes.{find, count, startswith, endswith}` ignored start/stop, and
# `bytes.rfind` was missing entirely. So:
#
#   b"abcabcabc".count(b"a", 0, 5)        # → 3   (wrong; should be 2)
#   b"abcabcabc".find(b"b", 3)            # → 1   (wrong; should be 4)
#   b"abcabcabc".rfind(b"b")              # AttributeError
#   b"abcabcabc".startswith(b"ab", 3)     # → True (start ignored)
#
# Fix in `runtime/bytes_ops.rs`:
#   - Introduce `clamp_range(len, start, end)` shared by all five —
#     CPython's defaults `0`/`len`, negatives count from the end and
#     floor at 0, positives cap at `len`, `end < start` collapses to
#     an empty slice.
#   - Add `_range` variants for find/count/startswith/endswith and a
#     fresh `mb_bytes_rfind` (right-search via `rposition`).
#   - Dispatcher routes 1-, 2-, 3-arg forms; the bare-arg helpers
#     forward through the new `_range` functions so existing call
#     sites stay byte-equivalent.

b = b"abcabcabc"

# Find with start/stop.
print(b.find(b"b"))                  # 1
print(b.find(b"b", 3))               # 4   (skip first b)
print(b.find(b"b", 0, 4))            # 1
print(b.find(b"b", 5, 9))            # 7
print(b.find(b"z"))                  # -1
print(b.find(b"z", 0, 100))          # -1
print(b.find(b""))                   # 0   (empty needle at start)
print(b.find(b"", 3))                # 3

# rfind — completely new.
print(b.rfind(b"b"))                 # 7
print(b.rfind(b"b", 0, 5))           # 4
print(b.rfind(b"b", 0, 4))           # 1
print(b.rfind(b"a"))                 # 6
print(b.rfind(b"z"))                 # -1
print(b.rfind(b""))                  # 9   (empty: returns end)

# Count with start/stop.
print(b.count(b"a"))                 # 3
print(b.count(b"a", 1))              # 2
print(b.count(b"a", 0, 5))           # 2
print(b.count(b"abc"))               # 3
print(b.count(b"abc", 1))            # 2
print(b.count(b""))                  # 10  (len(b)+1)
print(b.count(b"", 2, 5))            # 4   (len(slice)+1)

# Startswith / endswith with start/stop.
print(b.startswith(b"ab"))           # True
print(b.startswith(b"ab", 3))        # True   (slice "abcabc" starts with ab)
print(b.startswith(b"ab", 0, 2))     # True   (slice "ab" startswith "ab")
print(b.startswith(b"ab", 0, 1))     # False  (slice "a" too short)
print(b.startswith((b"x", b"a")))    # True
print(b.startswith((b"x", b"y"), 3)) # False

print(b.endswith(b"bc"))             # True
print(b.endswith(b"bc", 0, 5))       # False
print(b.endswith(b"bc", 0, 6))       # True
print(b.endswith((b"abc", b"xy")))   # True
print(b.endswith((b"x", b"y"), 0, 5))# False

# bytearray uses the same dispatcher → same fix applies.
ba = bytearray(b"abcabcabc")
print(ba.find(b"b", 3))              # 4
print(ba.count(b"a", 0, 5))          # 2
print(ba.startswith(b"ab", 3))       # True
print(ba.rfind(b"b", 0, 5))          # 4
