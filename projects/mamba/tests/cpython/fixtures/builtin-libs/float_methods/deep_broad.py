# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# float methods deep broad

# arithmetic
print(1.5 + 2.5)
print(2.5 - 1.5)
print(1.5 * 2.0)
print(3.0 / 2.0)
print(3.5 // 2.0)
print(3.5 % 2.0)
print(2.0 ** 3)

# negative
print(-1.5 + 0.5)
print(-1.5 * -2.0)

# signs
print(abs(-3.14))
print(abs(3.14))

# float constants
print(1e3)
print(1.5e2)
print(1.5e-2)
print(2e10)

# conversion
print(float(42))
print(float("3.14"))
print(float("-7.0"))
print(float(True))
print(float(False))

# int conversion truncates
print(int(3.7))
print(int(3.2))
print(int(-3.7))

# float comparisons
print(1.5 < 2.0)
print(1.5 == 1.5)
print(1.5 != 1.6)
print(0.1 + 0.2 == 0.3)  # False due to fp
print(abs((0.1 + 0.2) - 0.3) < 1e-10)  # True

# rounding
print(round(3.14159, 2))
print(round(3.14159, 4))
print(round(-3.5))
print(round(3.5))
print(round(4.5))

# min / max on floats
print(min(1.5, 2.5))
print(max(1.5, 2.5))
print(min([3.14, 1.0, 2.7]))

