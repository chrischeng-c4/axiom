# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# int() conversions from CPython test_int.py
print(int("42"))
print(int("0xff", 16))
print(int("0b1010", 2))
print(int("0o77", 8))
print(int(3.14))
print(int(-3.14))
print(int(True))
print(int(False))
# PEP 515 — digit-separator underscores in int() string parsing.
print(int("1_000_000"))
print(int("-1_234"))
print(int("+1_2_3"))
print(int("0x_FF", 16))
print(int("0b_1010", 2))
print(int("0o_777", 8))
# Invalid underscore placements raise ValueError.
for bad in ["1__000", "_100", "100_", "1_"]:
    try:
        int(bad)
        print("expected error for:", bad)
    except ValueError:
        print("rejected:", bad)
