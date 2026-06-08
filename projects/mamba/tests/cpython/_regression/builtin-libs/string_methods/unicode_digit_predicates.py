# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# `str.isdigit() / isdecimal() / isnumeric()` were ASCII-only:
#  - `isdecimal` checked `c.is_ascii_digit()` — wrong for Arabic-Indic ٠٩ /
#    Devanagari ०९ / fullwidth ０９, which have `General_Category=Nd`.
#  - `isdigit` was the same predicate — extra-wrong because `isdigit` is a
#    *superset* of isdecimal: it should also accept superscripts (²³⁴) and
#    single-glyph circled digits (①②) that have `Numeric_Type=Digit`.
#  - `isnumeric` used Rust's `c.is_numeric()` which already matches GC in
#    {Nd, Nl, No}, so it was incidentally close to right; pinned via
#    explicit `general_category()` check for clarity.
#
# Fix in `runtime/string_ops.rs`: pull in `unicode-properties` as a direct
# dep, route the three predicates through `c.general_category()`, and add an
# `is_unicode_digit_no` helper enumerating the No-with-`Nt=Digit` ranges
# (superscripts, subscripts, circled/parenthesized/full-stopped digits 0-9,
# dingbat negative-circled ones). Multi-digit decorated numerals like ⑩-⑳
# carry `Nt=Numeric` not `Nt=Digit`, so they remain `isdigit() == False`.

# Decimal_Number — all three predicates True.
print("0".isdecimal(), "0".isdigit(), "0".isnumeric())          # True True True
print("٠".isdecimal(), "٠".isdigit(), "٠".isnumeric())          # Arabic-Indic 0
print("१".isdecimal(), "१".isdigit(), "١".isnumeric())          # Devanagari 1
print("０".isdecimal(), "０".isdigit(), "０".isnumeric())       # fullwidth 0

# Numeric_Type=Digit but not Decimal — isdigit + isnumeric, NOT isdecimal.
print("²".isdecimal(), "²".isdigit(), "²".isnumeric())          # False True True
print("⁹".isdecimal(), "⁹".isdigit(), "⁹".isnumeric())          # superscript 9
print("₃".isdecimal(), "₃".isdigit(), "₃".isnumeric())          # subscript 3
print("①".isdecimal(), "①".isdigit(), "①".isnumeric())         # circled 1

# Numeric_Type=Numeric only — isnumeric only (fractions, Roman numerals,
# multi-digit decorated numerals).
print("½".isdecimal(), "½".isdigit(), "½".isnumeric())          # False False True
print("Ⅷ".isdecimal(), "Ⅷ".isdigit(), "Ⅷ".isnumeric())         # Roman 8
print("⑩".isdecimal(), "⑩".isdigit(), "⑩".isnumeric())         # circled 10 (Nu)

# Non-numeric — all False.
print("a".isdecimal(), "a".isdigit(), "a".isnumeric())          # False False False
print("".isdecimal(), "".isdigit(), "".isnumeric())              # empty -> all False
print("1.5".isdecimal(), "1.5".isdigit(), "1.5".isnumeric())    # '.' breaks all

# Mixed strings — all-or-nothing semantics.
print("12".isdecimal(), "12".isdigit(), "12".isnumeric())       # True True True
print("12a".isdecimal(), "12a".isdigit(), "12a".isnumeric())    # False False False
print("²³".isdecimal(), "²³".isdigit(), "²³".isnumeric())       # False True True
