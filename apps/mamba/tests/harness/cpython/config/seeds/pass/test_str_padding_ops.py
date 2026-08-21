# Operational AssertionPass seed for str padding, case-folding, and
# case-rotation methods not covered by test_str_ops.py.
# Surface: center, ljust, rjust with explicit fill chars; zfill;
# swapcase (inverts case per character); capitalize (Title-case the
# first char, lower the rest of the first word).
_ledger: list[int] = []
# center with custom fill — odd-length padding goes to the right
assert "hello".center(11, "*") == "***hello***"; _ledger.append(1)
# ljust pads on the right
assert "hello".ljust(10, ".") == "hello....."; _ledger.append(1)
# rjust pads on the left
assert "hello".rjust(10, ".") == ".....hello"; _ledger.append(1)
# zfill pads with leading zeros to the target width
assert "hello".zfill(10) == "00000hello"; _ledger.append(1)
assert "42".zfill(5) == "00042"; _ledger.append(1)
# swapcase inverts case per character (lower↔upper)
assert "HelloWorld".swapcase() == "hELLOwORLD"; _ledger.append(1)
# Digits are left untouched by swapcase
assert "Hello123World".swapcase() == "hELLO123wORLD"; _ledger.append(1)
# swapcase on empty string is empty
assert "".swapcase() == ""; _ledger.append(1)
# capitalize: Title-case the first char, lowercase the rest of word
assert "hello world".capitalize() == "Hello world"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_str_padding_ops {sum(_ledger)} asserts")
