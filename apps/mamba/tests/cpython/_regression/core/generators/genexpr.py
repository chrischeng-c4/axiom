# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Generator expressions: basic, filtered, nested, as function arg (R1)
print(list(x ** 2 for x in range(5)))
print(list(x for x in range(10) if x % 2 == 0))
print(sum(x for x in range(4)))
# Nested generator expression
print(list((x, y) for x in range(3) for y in range(2)))
# As function argument
print(max(x ** 2 for x in range(-3, 4)))
print(min(abs(x) for x in [-5, 3, -1, 4]))
