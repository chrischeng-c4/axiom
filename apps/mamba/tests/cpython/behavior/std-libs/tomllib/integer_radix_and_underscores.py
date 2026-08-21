# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "behavior"
# case = "integer_radix_and_underscores"
# subject = "tomllib.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_data.py"
# status = "filled"
# ///
"""tomllib.loads: integers parse in decimal, hex (0xFF=255), octal (0o77=63), binary (0b1010=10), signed, and with underscore separators (1_000_000)"""
import tomllib

_d = tomllib.loads("""
decimal = 42
negative = -17
hex_val = 0xFF
octal_val = 0o77
binary_val = 0b1010
underscore = 1_000_000
""")
assert _d["decimal"] == 42, f"decimal = {_d['decimal']!r}"
assert _d["negative"] == -17, f"negative = {_d['negative']!r}"
assert _d["hex_val"] == 255, f"hex = {_d['hex_val']!r}"
assert _d["octal_val"] == 63, f"octal = {_d['octal_val']!r}"
assert _d["binary_val"] == 10, f"binary = {_d['binary_val']!r}"
assert _d["underscore"] == 1000000, f"underscore = {_d['underscore']!r}"

print("integer_radix_and_underscores OK")
