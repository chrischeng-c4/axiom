# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# type / conversions / builtins

# isinstance broad
print(isinstance(True, int))
print(isinstance(1, bool))
print(isinstance(5, (int, str)))
print(isinstance("a", (int, str)))
print(isinstance(5.0, (int, str)))

# numeric conversions
print(int(3.7))
print(int(-3.7))
print(int("42"))
print(int("-42"))
print(int("0x10", 16))
print(int("101", 2))
print(float(3))
print(float("3.14"))
print(float("-1e3"))
print(str(42))
print(str(3.14))
print(str(True))
print(str(None))
print(bool(0))
print(bool(1))
print(bool(""))
print(bool("x"))
print(bool([]))
print(bool([0]))

# chr / ord
print(chr(65))
print(chr(97))
print(ord("A"))
print(ord("a"))
print(ord("0"))

# abs / round
print(abs(-5))
print(abs(5))
print(abs(-3.14))
print(round(3.7))
print(round(3.4))
print(round(3.14159, 2))
print(round(2.5))
print(round(3.5))

# divmod
print(divmod(17, 5))
print(divmod(-17, 5))
print(divmod(10, 3))

# min/max on mixed
print(min(5, 3, 8, 1))
print(max(5, 3, 8, 1))
print(min([5, 3, 8, 1]))
print(max([5, 3, 8, 1]))
print(min("banana"))
print(max("banana"))
