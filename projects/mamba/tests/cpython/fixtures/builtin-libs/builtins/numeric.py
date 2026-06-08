# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Builtins conformance: numeric functions (R5).
# abs, round, pow, divmod, int, float, complex
print(abs(-5))
print(abs(3.14))
print(abs(0))
print(abs(-0.0))
print(round(3.5))
print(round(4.5))
print(round(2.675, 2))
print(round(-3.5))
print(round(1234, -2))
print(pow(2, 10))
print(pow(3, 3))
print(pow(2, -1))
print(divmod(17, 5))
print(divmod(-7, 3))
print(divmod(7, -3))
print(int(3.9))
print(int(-3.9))
print(int(True))
print(int(False))
print(int("42"))
print(int("  -7  "))
print(float(42))
print(float(True))
print(float(False))
print(float("3.14"))
