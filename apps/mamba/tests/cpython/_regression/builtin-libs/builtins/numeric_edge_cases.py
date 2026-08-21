# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Numeric builtin edge cases: pow with modulus, int() with base
# Numeric builtin edge cases: abs, round, divmod, pow, int, float
print(abs(-0.0))
print(abs(float('inf')))
print(round(2.5))
print(round(3.5))
print(divmod(7, 3))
print(pow(2, 10))
print(pow(2, -1))
print(pow(2, 10, 1000))
print(int('0xff', 16))
print(float('inf'))
print(float('-inf'))
print(bin(255))
print(hex(255))
print(oct(255))
print(bin(-1))
print(hex(-1))
