# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""int() parsing of string / bytes input: whitespace stripping, unicode
digit scripts, bytes-like sources, and the inputs that must be rejected.
"""

# Leading / trailing ASCII and unicode whitespace is stripped.
assert int("  \t\t  314  \t\t  ") == 314
assert int(" -3 ") == -3   # EM SPACE / EN SPACE around the value
assert int(" -3 ") == -3
print("whitespace: ok")

# Decimal digits from non-Latin scripts are accepted.
assert int("١٢٣٤٥٦٧٨٩٠") == 1234567890                 # Arabic-Indic
assert int("१२३४५६७८९०1234567890") == 12345678901234567890  # Devanagari + ASCII
assert int("١٢٣٤٥٦٧٨٩٠", 0) == 1234567890
print("unicode_digits: ok")

# bytes and bytearray are valid sources, parsed as ASCII text.
assert int(b"10") == 10
assert int(b"-1") == -1
assert int(bytearray(b"42")) == 42
assert int(b"1_00") == 100
print("bytes_like: ok")

# A memoryview (including a slice of one) is parsed too.
assert int(memoryview(b"123")[1:3]) == 23
assert int(memoryview(b"1234")[1:3]) == 23
print("memoryview: ok")

# A bare or detached sign is invalid; a space after the sign is invalid.
for bad in ("+", "-", "- 1", "+ 1", " + 1 ", "_100", "+_100"):
    try:
        int(bad)
        print("invalid_sign: no_raise", repr(bad))
        break
    except ValueError:
        pass
else:
    print("invalid_sign: ValueError")

# A float-shaped string is never an integer.
for bad in ("1.2", "1x", "", " ", "  1\x02  "):
    try:
        int(bad)
        print("float_str: no_raise", repr(bad))
        break
    except ValueError:
        pass
else:
    print("float_str: ValueError")

print("int_string_parsing OK")
