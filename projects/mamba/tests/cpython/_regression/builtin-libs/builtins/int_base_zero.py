# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# `int(s, 0)` panicked outright: `mb_int_base` forwarded `base_num` to
# `i64::from_str_radix`, which rejects `radix == 0` with a panic.
#
#   thread 'main' panicked at .../num/mod.rs:1666:1:
#   from_ascii_radix: radix must lie in the range `[2, 36]` - found 0
#
# CPython's contract for `int(s, 0)` is "auto-detect from the prefix":
# `0x` / `0X` → 16, `0o` / `0O` → 8, `0b` / `0B` → 2, otherwise base 10.
# When the string has no recognised prefix, leading zeros are forbidden
# (so `int("010", 0)` raises ValueError, matching the literal-syntax
# rule that motivated PEP 3127).
#
# Fix in `runtime/builtins.rs::mb_int_base`: when `base_num == 0`,
# pre-detect the prefix (and its bare-decimal "no leading zeros"
# rule), pick an `effective_base`, then proceed through the existing
# strip/parse path.

# Hex.
print(int("0xff", 0))                       # 255
print(int("0XFF", 0))                       # 255
print(int("+0xff", 0))                      # 255
print(int("-0xff", 0))                      # -255
print(int("  0xff  ", 0))                   # 255   (whitespace allowed)

# Octal.
print(int("0o17", 0))                       # 15
print(int("0O17", 0))                       # 15
print(int("-0o10", 0))                      # -8

# Binary.
print(int("0b101", 0))                      # 5
print(int("0B101", 0))                      # 5
print(int("-0b1", 0))                       # -1

# Plain decimal (no leading zeros).
print(int("42", 0))                         # 42
print(int("-10", 0))                        # -10
print(int("0", 0))                          # 0
print(int("+7", 0))                         # 7

# Leading zeros without a prefix → ValueError (CPython).
try:
    int("010", 0)
except ValueError as e:
    print("VE:", e)
try:
    int("00", 0)
except ValueError as e:
    print("VE-double-zero:", e)

# Non-digit garbage still raises rather than panicking.
try:
    int("abc", 0)
except ValueError as e:
    print("VE-garbage:", e)
try:
    int("0xZZ", 0)
except ValueError as e:
    print("VE-bad-hex:", e)

# Existing explicit-base paths keep working unchanged.
print(int("0xff", 16))                      # 255
print(int("ff", 16))                        # 255
print(int("ff", 16) + int("11", 16))        # 272
print(int("101", 2))                        # 5
print(int("42"))                            # 42
