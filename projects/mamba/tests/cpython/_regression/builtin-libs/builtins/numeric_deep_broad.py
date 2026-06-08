# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# numeric builtins broad

# abs
print(abs(5))
print(abs(-5))
print(abs(0))
print(abs(1.5))
print(abs(-1.5))
print(abs(-0.0))

# pow
print(pow(2, 10))
print(pow(3, 4))
print(pow(10, 0))
print(pow(2, -1))
print(pow(2, 3, 5))
print(pow(7, 2, 10))

# divmod
print(divmod(10, 3))
print(divmod(20, 7))
print(divmod(100, 10))
print(divmod(-10, 3))

# round
print(round(3.14))
print(round(3.7))
print(round(3.5))
print(round(2.5))
print(round(-3.5))
print(round(3.14159, 2))
print(round(3.14159, 4))
print(round(123.456, 1))

# min/max on multiple args
print(min(3, 1, 2))
print(max(3, 1, 2))
print(min(1, 2, 3, 4, 5))
print(max(1, 2, 3, 4, 5))

# any / all
print(any([1, 0, 0]))
print(any([0, 0, 0]))
print(any([]))
print(all([1, 1, 1]))
print(all([1, 0, 1]))
print(all([]))

# any on strings
print(any([""]))
print(any(["", "x"]))
print(all(["a", "b"]))
print(all(["a", ""]))

# any on bools
print(any([False, True, False]))
print(all([True, True, True]))

# sum
print(sum([1, 2, 3, 4, 5]))
print(sum([]))
print(sum([1, 2, 3], 10))  # start
print(sum([1.5, 2.5, 3.0]))

# len on common
print(len([1, 2, 3]))
print(len("hello"))
print(len((1, 2, 3, 4)))
print(len({1, 2, 3}))
print(len({"a": 1, "b": 2}))
print(len(""))
print(len([]))

# chr / ord
print(ord("a"))
print(ord("A"))
print(ord("0"))
print(ord(" "))
print(chr(65))
print(chr(97))
print(chr(48))
print(chr(32))

# str/int/float conv
print(str(42))
print(str(3.14))
print(str(True))
print(str([1, 2, 3]))
print(int("42"))
print(int("-5"))
print(int(3.7))
print(int(-3.7))
print(float("3.14"))
print(float(42))
print(float("-2.5"))
