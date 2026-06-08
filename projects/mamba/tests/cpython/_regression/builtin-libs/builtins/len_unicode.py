# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Python 3 `len(str)` returns the number of Unicode code points, not the
# UTF-8 byte length. Mamba was returning `s.len()` from Rust (byte length),
# which counted accented chars and emoji as multiple "characters". Fix in
# `runtime/builtins.rs::mb_len` switches to `s.chars().count()`.

print(len(""))                       # 0
print(len("ascii"))                  # 5
print(len("héllo"))                  # 5 (was 6 — é = 2 UTF-8 bytes)
print(len("café"))                   # 4 (was 5)
print(len("日本語"))                  # 3 (was 9 — each char = 3 bytes)
print(len("🎉"))                      # 1 (was 4 — 4-byte UTF-8 sequence)
print(len("a🎉b"))                    # 3 (was 6)
print(len("hello\tworld"))            # 11

# bytes still measured in bytes (a separate code path in mb_len).
print(len(b"hello"))                  # 5
