# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "bigint_literal_parsing"
# subject = "integer literals"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Integer literals larger than i64 parse as Python arbitrary-precision ints."""

decimal_value = 123456789012345678901234567890
assert decimal_value == int("123456789012345678901234567890")
assert repr(decimal_value) == "123456789012345678901234567890"

hex_value = 0x1_0000_0000_0000_0000
assert hex_value == int("10000000000000000", 16)

binary_value = 0b1_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000
assert binary_value == int("10000000000000000000000000000000000000000000000000000000000000000", 2)

octal_value = 0o1_0000_0000_0000_0000_0000_0000
assert octal_value == int("1000000000000000000000000", 8)

assert -9223372036854775808 == -int("9223372036854775808")

print("bigint_literal_parsing OK")
